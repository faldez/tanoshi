FROM faldez/tanoshi-builder:latest AS planner
COPY crates/ /app/crates/
COPY Cargo.lock /app/Cargo.lock
COPY Cargo.toml /app/Cargo.toml
RUN cargo chef prepare  --recipe-path recipe.json

FROM faldez/tanoshi-builder:latest AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo install trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
RUN cargo chef cook --release --recipe-path recipe.json

COPY crates/ /app/crates/
COPY Cargo.lock /app/Cargo.lock
COPY Cargo.toml /app/Cargo.toml

RUN cd crates/tanoshi-web; trunk build

RUN cargo build -p tanoshi --release

FROM debian:stable-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/tanoshi .
RUN chmod +x tanoshi

RUN apt update && apt upgrade -y && apt install --reinstall -y ca-certificates

ENV TANOSHI_LOG=info
ENV TANOSHI_HOME=/tanoshi

EXPOSE 80

ENTRYPOINT ["/app/tanoshi"]
