FROM rust:latest as builder
WORKDIR /opt/rss-rewrite
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
EXPOSE 8000
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /opt/rss-rewrite/target/release/rss-rewrite /usr/local/bin/rss-rewrite
CMD ["rss-rewrite", "/feeds.json"]
