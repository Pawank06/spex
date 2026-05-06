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

## Usage

In one terminal, start the receiver:

```sh
cargo run -p spex-cli -- recv --bind 127.0.0.1:7002 --peer 127.0.0.1:7001 --out out.bin
```

In another, send a file:

```sh
cargo run -p spex-cli -- send --bind 127.0.0.1:7001 --peer 127.0.0.1:7002 --file in.bin
```

Use `-v` for info logs and `-vv` for debug logs.

## Testing

```sh
cargo test --all
```

## License

MIT — see [LICENSE](LICENSE).
