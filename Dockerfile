# Build miniping for Linux x86_64 using musl for static linking
FROM rust:alpine AS builder

# Install musl target
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build static binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/miniping /miniping
ENTRYPOINT ["/miniping"]