[![Build and test](https://github.com/antonilol/rust-bitcoincore-zmq/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/antonilol/rust-bitcoincore-zmq/actions/workflows/build_and_test.yml)
[![Integration tests](https://github.com/antonilol/rust-bitcoincore-zmq/actions/workflows/integration_tests.yml/badge.svg)](https://github.com/antonilol/rust-bitcoincore-zmq/actions/workflows/integration_tests.yml)
[![crates.io](https://img.shields.io/crates/v/bitcoincore-zmq.svg)](https://crates.io/crates/bitcoincore-zmq)

# Tokio Bitcoin Core ZMQ

This is a fork of [`bitcoincore-zmq`](https://crates.io/crates/bitcoincore-zmq). The goal is to provide a `tokio`-compatible implementation of a [Bitcoin Core ZMQ](https://github.com/bitcoin/bitcoin/blob/master/doc/zmq.md) subscriber.

### Usage example
```rust
#[tokio::main]
async fn main() ->  Result<(), Box<dyn Error>> {
    let zmq_url = "tcp://127.0.0.1:28332";
    let mut zmq = bitcoincore_zmq::subscribe_single(&zmq_url).await?;

    while let Some(msg) = zmq.recv().await {
        match msg {
            Ok(msg) => println!("Received message: {msg}"),
            Err(err) => println!("Error receiving message: {err}"),
        }
    }
}
```

### Testing

Tests run on every push and pull request.
Integration tests use the latest version of the 3 most recent major Bitcoin Core versions, see [integration_tests.yml](.github/workflows/integration_tests.yml#L19-L21).

---

TODO:
- This README
- Message test
- SequenceMessage itest
- Easy addEventListener like functionality with help of the `getzmqnotifications` rpc (bitcoincore-rpc PR: #295)
- raw messages
- zmq publisher
