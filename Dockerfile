FROM rust:1.93.0 AS builder
WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./

# compile all dependencies with a dummy for improved caching
RUN mkdir -p src/bin && \
  echo "fn main() { println!(\"Dummy\"); }" > src/bin/dummy.rs && \
  sed -i -E s/^default-run\.\+// Cargo.toml && \
  cargo build --release --bin dummy && \
  rm -rf src

# now compile the real code
COPY src src
RUN cargo install --locked --path . --root /usr/local
RUN cargo run --release --bin crdgen > crds.yaml

FROM debian:stable-slim

RUN useradd --system --no-create-home --shell /usr/bin/nologin m7o
COPY --from=builder /usr/local/bin/m7o /usr/local/bin

USER m7o
WORKDIR /m7o

COPY --from=builder /usr/src/crds.yaml /m7o/crds.yaml

CMD [ "m7o" ]
