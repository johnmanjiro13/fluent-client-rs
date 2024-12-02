use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result as AnyhowResult;
use async_trait::async_trait;
use bytes::{Buf, BufMut};
use log::{debug, warn};
use rmp_serde::Serializer;
use serde::{ser::SerializeMap, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixStream},
    sync::broadcast::{error::RecvError, Receiver},
    time::{timeout, Duration},
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

#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub tag: String,
    pub timestamp: i64,
    pub record: Map,
    pub options: Options,
}

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub enum Message {
    Record(Record),
    Terminate,
}

#[derive(Debug)]
struct SerializedRecord {
    record: bytes::Bytes,
    chunk: String,
}

#[derive(Debug, Deserialize)]
struct AckResponse {
    ack: String,
}

pub struct RetryConfig {
    pub initial_wait: u64,
    pub max: u32,
    pub max_wait: u64,
}

pub struct Worker<StreamType> {
    stream_config: Arc<dyn Connectable<StreamType> + Send + Sync>,
    max_connection_lifetime: Duration,
    stream: Cell<StreamType>,
    last_connection_time: Cell<Instant>,
    receiver: Receiver<Message>,
    retry_config: RetryConfig,
}

impl<StreamType> Worker<StreamType>
where
    StreamType: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub async fn new(
        stream_config: Arc<dyn Connectable<StreamType> + Send + Sync>,
        max_connection_lifetime: Duration,
        receiver: Receiver<Message>,
        retry_config: RetryConfig,
    ) -> AnyhowResult<Self> {
        let stream = stream_config.connect().await?;
        Ok(Self {
            stream_config,
            max_connection_lifetime,
            stream: Cell::new(stream),
            last_connection_time: Cell::new(Instant::now()),
            receiver,
            retry_config,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.recv().await {
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
                Err(RecvError::Closed) | Ok(Message::Terminate) => {
                    break;
                }
                Err(RecvError::Lagged(_)) => continue,
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

            // reconnect when the lifetime is reached
            if !self.max_connection_lifetime.is_zero()
                && self.last_connection_time.get().elapsed() >= self.max_connection_lifetime
            {
                debug!("attempting to re-establish connection");
                match self.stream_config.connect().await {
                    Ok(new_stream) => {
                        self.stream.replace(new_stream);
                        self.last_connection_time.replace(Instant::now());
                    }
                    Err(err) => {
                        warn!(
                            "failed to reconnect. Will try again upon the next try-write: {}",
                            err
                        );
                    }
                }
            }

            match Self::write(&mut self.stream.get_mut(), record).await {
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

    async fn write(stream: &mut StreamType, record: &SerializedRecord) -> Result<(), Error> {
        stream
            .write_all(record.record.chunk())
            .await
            .map_err(|e| Error::WriteFailed(e.to_string()))?;

        let received_ack = Self::read_ack(stream).await?;

        if received_ack.ack != record.chunk {
            warn!(
                "ack and chunk did not match. ack: {}, chunk: {}",
                received_ack.ack, record.chunk
            );
            return Err(Error::AckUnmatched(received_ack.ack, record.chunk.clone()));
        }
        Ok(())
    }

    async fn read_ack(stream: &mut StreamType) -> Result<AckResponse, Error> {
        let mut buf = bytes::BytesMut::with_capacity(64);
        loop {
            if let Ok(ack) = rmp_serde::from_slice::<AckResponse>(&buf) {
                return Ok(ack);
            }

            if stream
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

#[async_trait]
pub trait Connectable<T> {
    async fn connect(&self) -> AnyhowResult<T>;
}

#[derive(Debug)]
pub struct TCPConnectionConfig {
    pub addr: std::net::SocketAddr,
    pub timeout: Duration,
}

#[async_trait]
impl Connectable<TcpStream> for TCPConnectionConfig {
    async fn connect(&self) -> AnyhowResult<TcpStream> {
        let stream = timeout(self.timeout, TcpStream::connect(self.addr)).await??;
        Ok(stream)
    }
}

#[derive(Debug)]
pub struct UnixSocketConfig {
    pub path: PathBuf,
    pub timeout: Duration,
}

#[async_trait]
impl Connectable<UnixStream> for UnixSocketConfig {
    async fn connect(&self) -> AnyhowResult<UnixStream> {
        let stream = timeout(self.timeout, UnixStream::connect(self.path.as_path())).await??;
        Ok(stream)
    }
}
