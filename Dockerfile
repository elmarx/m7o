FROM rust:1.87.0 AS builder
WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./

# compile all dependencies with a dummy for improved caching
RUN mkdir -p src/bin && \
  echo "fn main() { println!(\"Dummy\"); }" > src/main.rs && \
  cargo build --release && \
  rm -rf src

COPY src src
RUN touch src/main.rs && \
    cargo install --locked --path . --root /usr/local

FROM debian:stable-slim

RUN useradd --system --no-create-home --shell /usr/bin/nologin m7o
USER m7o

EXPOSE 8080
CMD [ "m7o" ]

COPY --from=builder /usr/local/bin/m7o /usr/local/bin
