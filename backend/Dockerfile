# 1. Build Stage
FROM rust:1.81 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 2. Run Stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /app/server

# Run the binary
CMD ["./server"]