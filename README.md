# unproxy

Map TCP services behind an HTTP(S) proxy to local ports.

## Usage (CLI)

```
cargo install unproxy-cli
unproxy-cli --proxy https://proxy.example.com --local 127.0.0.1:5432 --remote 127.0.0.1:5432
```

## Usage (library)

[Docs](https://docs.rs/unproxy)

```rust
let proxy_stream = unproxy::connect(proxy_url, remote_hostname, remote_port).await?;
```
