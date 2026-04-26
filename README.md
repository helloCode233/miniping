# miniping

A minimal ping utility written in Rust, inspired by fping.

## Features

- Send ICMP echo requests to a single host
- Configurable count, timeout, and interval
- Works on Linux with DGRAM sockets (no root required if `ping_group_range` is set)
- IPv4 and IPv6 support

## Build profiles

| Profile | Command | Use case | Size |
|---------|---------|----------|------|
| dev     | `cargo build` | Fast iteration, debug info | ~3.2M |
| release | `cargo build --release` | Balanced speed/size | ~950K |
| production | `cargo build --profile production` | **Maximum size reduction** | ~640K |

## Building

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- For Linux cross‑compilation: `brew install FiloSottile/musl-cross/musl-cross`

### macOS (dev)

```bash
cargo build                # debug binary
cargo build --release      # release binary
```

### Linux x86_64 — development build

```bash
rustup target add x86_64-unknown-linux-musl
make linux-release
# or manually:
cargo build --release --target x86_64-unknown-linux-musl
```

Binary at `miniping-linux-x86_64-release`.

### Linux x86_64 — production build (recommended)

```bash
make linux-production
# or manually:
cargo build --profile production --target x86_64-unknown-linux-musl
# or via script:
./build-linux.sh production
```

Binary at `miniping-linux-x86_64` (~640K, static, stripped).

### Type of build (balanced speed/size):

```bash
./build-linux.sh release   # → miniping-linux-x86_64 (~800K)
```

## Usage

```
miniping [OPTIONS] <HOST>

Arguments:
  <HOST>  Target host to ping

Options:
  -c, --count <COUNT>      Number of echo requests [default: 4]
  -t, --timeout <TIMEOUT>  Timeout in seconds [default: 1]
  -i, --interval <INTERVAL>  Interval between requests in ms [default: 1000]
  -h, --help               Print help
```

Example:

```bash
miniping 8.8.8.8 -c 5
```

## Notes

- On Linux, non‑root users can ping if their GID is within the range defined in `/proc/sys/net/ipv4/ping_group_range`. To enable, run as root:
  ```bash
  echo "0 1000" > /proc/sys/net/ipv4/ping_group_range
  ```
  (adjust the range as needed).

- The program uses `SOCK_DGRAM` ICMP sockets, which are supported on Linux and macOS.

## License

MIT