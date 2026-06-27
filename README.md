# smart-home-network

Network version of the smart home example.

## Binaries

Run the TCP socket simulator:

```bash
cargo run --bin socket-simulator -- 127.0.0.1:4000
```

Run the UDP thermometer simulator with a config file:

```text
target_addr = 127.0.0.1:4001
period_ms = 500
```

```bash
cargo run --bin thermometer-simulator -- thermometer.conf
```

Run the smart home example against already started simulators:

```bash
cargo run --bin smart-home-network
```

For a self-contained demo:

```bash
cargo run --bin smart-home-network -- --start-simulators
```
