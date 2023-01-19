use bytes::{Buf, BufMut};
use crossbeam::channel::{self, Receiver};
use log::warn;
use rmp_serde::Serializer;
use serde::{ser::SerializeMap, Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter},
    net::TcpStream,
    time::Duration,
};

use crate::record::Map;

const RETRY_INCREMENT_RATE: f64 = 1.5;

#[derive(Debug, Clone)]
pub enum Error {
    WriteFailed(String),
    ReadFailed(String),
    AckUnmatched(String, String),
    MaxRetriesExceeded,
    ConnectionClosed,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Error::WriteFailed(ref e) => e,
            Error::ReadFailed(ref e) => e,
            Error::AckUnmatched(_, _) => "request chunk and response ack did not match",
            Error::MaxRetriesExceeded => "max retries exceeded",
            Error::ConnectionClosed => "connection closed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize)]
pub struct Record {
    pub tag: &'static str,
    pub timestamp: i64,
    pub record: Map,
    pub options: Options,
}

#[derive(Debug)]
pub struct Options {
    pub chunk: String,
}

impl Serialize for Options {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("chunk", &self.chunk)?;
        map.end()
    }
}

pub enum Message {
    Record(Record),
    Terminate,
}

#[derive(Debug)]
struct SerializedRecord {
    record: bytes::Bytes,
    chunk: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AckResponse {
    ack: String,
}

pub struct RetryConfig {
    pub initial_wait: u64,
    pub max: u32,
    pub max_wait: u64,
}

pub struct Worker<T = TcpStream> {
    stream: BufWriter<T>,
    receiver: Receiver<Message>,
    retry_config: RetryConfig,
}

impl<T> Worker<T>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    pub fn new(stream: T, receiver: Receiver<Message>, retry_config: RetryConfig) -> Self {
        Self {
            stream: BufWriter::new(stream),
            receiver,
            retry_config,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(Message::Record(record)) => {
                    let record = match self.encode(record) {
                        Ok(record) => record,
                        Err(e) => {
                            warn!("failed to serialize a message: {}", e);
                            continue;
                        }
                    };

                    match self.write_with_retry(&record).await {
                        Ok(_) => {}
                        Err(_) => continue,
                    };
                }
                Err(channel::TryRecvError::Empty) => continue,
                Ok(Message::Terminate) | Err(channel::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn encode(&self, record: Record) -> Result<SerializedRecord, rmp_serde::encode::Error> {
        let mut writer = bytes::BytesMut::new().writer();
        record.serialize(&mut Serializer::new(&mut writer))?;
        Ok(SerializedRecord {
            record: writer.into_inner().freeze(),
            chunk: record.options.chunk,
        })
    }

    async fn write_with_retry(&mut self, record: &SerializedRecord) -> Result<(), Error> {
        let mut wait_time = Duration::from_millis(0);
        for i in 0..self.retry_config.max as i32 {
            tokio::time::sleep(wait_time).await;

            match self.write(record).await {
                Ok(_) => return Ok(()),
                Err(Error::ConnectionClosed) => return Err(Error::ConnectionClosed),
                Err(_) => {}
            }

            let mut t =
                (self.retry_config.initial_wait as f64 * RETRY_INCREMENT_RATE.powi(i - 1)) as u64;
            if t > self.retry_config.max_wait {
                t = self.retry_config.max_wait;
            }
            wait_time = Duration::from_millis(t);
        }
        warn!("write's max retries exceeded.");
        Err(Error::MaxRetriesExceeded)
    }

    async fn write(&mut self, record: &SerializedRecord) -> Result<(), Error> {
        self.stream
            .write_all(record.record.chunk())
            .await
            .map_err(|e| Error::WriteFailed(e.to_string()))?;

        let received_ack = self.read_ack().await?;

        if received_ack.ack != record.chunk {
            warn!(
                "ack and chunk did not match. ack: {}, chunk: {}",
                received_ack.ack, record.chunk
            );
            return Err(Error::AckUnmatched(received_ack.ack, record.chunk.clone()));
        }
        Ok(())
    }

    async fn read_ack(&mut self) -> Result<AckResponse, Error> {
        let mut buf = bytes::BytesMut::with_capacity(64);
        loop {
            if let Ok(ack) = rmp_serde::from_slice::<AckResponse>(&buf) {
                return Ok(ack);
            }

            if self
                .stream
                .read_buf(&mut buf)
                .await
                .map_err(|e| Error::ReadFailed(e.to_string()))?
                == 0
            {
                return Err(Error::ConnectionClosed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_ack() {
        let (client, mut server) = tokio::io::duplex(64);
        let (_, receiver) = channel::unbounded();
        let retry_config = RetryConfig {
            initial_wait: 50,
            max: 13,
            max_wait: 60000,
        };
        let mut worker = Worker::new(client, receiver, retry_config);

        let ack_response = AckResponse {
            ack: "Mzc4NDQwMzctNGY4Ni00MmI2LWFiYjMtMjk3MGZkNDUzY2Y2".to_string(),
        };
        let ack_response = rmp_serde::to_vec(&ack_response).unwrap();
        server.write_all(&ack_response).await.unwrap();

        let ack = worker.read_ack().await.expect("failed to read ack");
        assert_eq!(
            ack.ack,
            "Mzc4NDQwMzctNGY4Ni00MmI2LWFiYjMtMjk3MGZkNDUzY2Y2".to_string()
        );
    }
}
