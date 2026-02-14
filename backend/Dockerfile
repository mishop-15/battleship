# 1. Build Stage
FROM rust:1.81 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 2. Run Stage (Small, clean image)
FROM debian:bookworm-slim
WORKDIR /app
# Install OpenSSL (needed for many Rust web request crates)
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/battleship_backend /app/battleship_backend

# Render usually sets the PORT env var, but we default to 3000
EXPOSE 3000

# Run the binary
CMD ["./battleship_backend"]