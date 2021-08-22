FROM rust:latest AS builder

WORKDIR /app

RUN apt update && \
    apt upgrade -y && \
    apt-get install -y \
    libssl-dev \
    libarchive-dev \
    build-essential \
    cmake \
    llvm \
    clang \
    libicu-dev \
    nettle-dev \
    libacl1-dev \
    liblzma-dev \
    libzstd-dev \
    liblz4-dev \
    libbz2-dev \
    zlib1g-dev \
    libxml2-dev
RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.13.1/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C $HOME/.cargo/bin
RUN chmod +x $HOME/.cargo/bin/trunk
RUN rustup target add wasm32-unknown-unknown

COPY . .

RUN cd tanoshi-web && trunk build --release
RUN cargo build -p tanoshi --release

FROM debian:buster-slim

WORKDIR /app

COPY --from=builder /app/target/release/tanoshi .
RUN chmod +x tanoshi

RUN apt update && apt upgrade -y && apt install --reinstall -y ca-certificates

ENV TANOSHI_LOG=info
ENV TANOSHI_HOME=/tanoshi

EXPOSE 80

ENTRYPOINT ["/app/tanoshi"]