# spex

`spex` — streaming protocol experiment.

A toy file-transfer protocol over UDP that demonstrates chunking, hashing,
out-of-order delivery, and selective retransmission.

## Crates

- `spex-core` — chunking, hashing, file metadata, and reassembly
- `spex-net` — sender and receiver tasks built on top of `tokio::net::UdpSocket`
- `spex-cli` — `spex` command line entry point

## Building

```sh
cargo build
```
