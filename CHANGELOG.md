# Changelog

## [v0.5.1](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.5.0...v0.5.1) - 2024-12-02
- feat: support max_connection_lifetime and reconnection by @danielsig727 in https://github.com/johnmanjiro13/tokio-fluent/pull/55
- fix(deps): update rust crate base64 to 0.22.0 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/51

## [v0.5.0](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.4.4...v0.5.0) - 2023-11-17
- !feat(client): uds support by @danielsig727 in https://github.com/johnmanjiro13/tokio-fluent/pull/47
### Breaking Changes
- Removed `new` method from `Client` and added `new_tcp` and `new_unix` methods.
  - Removed `addr` option from `Config`.


```rust
// Before
let client = Client::new(&Config {
    addr: "127.0.0.1:24224".parse().unwrap(),
    ..Default::default()
})

// After
let client = Client::new_tcp(
    "127.0.0.1:24224".parse().unwrap(),
    &Config {..Default::default()}
)
```

## [v0.4.4](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.4.3...v0.4.4) - 2023-10-29
- chore(deps): update actions/checkout action to v4 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/43
- fix(deps): update rust crate uuid to 1.5.0 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/44
- fix(deps): update rust crate base64 to 0.21.5 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/39

## [v0.4.3](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.4.2...v0.4.3) - 2023-08-15
- fix(deps): update rust crate serde to 1.0.183 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/33
- fix(deps): update rust crate log to 0.4.20 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/36
- chore(deps): update rust crate tokio to 1.31.0 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/35

## [v0.4.2](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.4.1...v0.4.2) - 2023-07-24
- fix(deps): update rust crate rmp-serde to 1.1.2 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/30
- fix(deps): update rust crate serde to 1.0.175 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/31

## [v0.4.1](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.4.0...v0.4.1) - 2023-07-18
- fix(deps): update rust crate uuid to 1.4.1 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/20
- chore(deps): update rust crate tokio to 1.29.1 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/18
- fix(deps): update rust crate serde to 1.0.171 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/21
- fix(deps): update rust crate log to 0.4.19 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/24
- fix(deps): update rust crate bytes to 1.4.0 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/19
- fix(deps): update rust crate chrono to 0.4.26 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/22
- fix(deps): update rust crate base64 to 0.21.2 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/23

## [v0.4.0](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.3.1...v0.4.0) - 2023-07-18
- ci: Run test with multiple fluentd versions by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/27
- Allow for regular strings instead of only static strings by @DaanDD in https://github.com/johnmanjiro13/tokio-fluent/pull/25
- Use tokio streams instead of crossbeam to fix deadlock issues by @DaanDD in https://github.com/johnmanjiro13/tokio-fluent/pull/26

## [v0.3.1](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.3.0...v0.3.1) - 2023-01-20
- Revert "test: read_ack" by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/15

## [v0.3.0](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.2.1...v0.3.0) - 2023-01-19
- Add timeout option by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/10
- Add retry option by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/11
- Use only necessary feature and add tests by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/13
- chore(deps): update rust crate tokio to 1.24.2 by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/3
- chore: Update README by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/14

## [v0.2.1](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.2.0...v0.2.1) - 2023-01-08
- fix: Continue when some error occurred by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/1
- Configure Renovate by @renovate in https://github.com/johnmanjiro13/tokio-fluent/pull/2
- chore: Configure renovate by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/5
- ci: Configure tagpr by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/6
- ci: Cargo publish after tagpr is merged by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/8
- Fix ci by @johnmanjiro13 in https://github.com/johnmanjiro13/tokio-fluent/pull/9

## [v0.2.0](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.1.2...v0.2.0) - 2023-01-07

## [v0.1.2](https://github.com/johnmanjiro13/tokio-fluent/compare/v0.1.1...v0.1.2) - 2023-01-07

## [v0.1.1](https://github.com/johnmanjiro13/tokio-fluent/commits/v0.1.1) - 2023-01-07
