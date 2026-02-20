FROM rust:1.84 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY db ./db
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/rust-acl-service /usr/local/bin/rust-acl-service
COPY db ./db
EXPOSE 8080 9000
CMD ["rust-acl-service"]
