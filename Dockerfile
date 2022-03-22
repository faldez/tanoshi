# Frontend planner
FROM --platform=$BUILDPLATFORM faldez/tanoshi-builder:latest AS web-planner

COPY crates/tanoshi-web .

RUN cargo chef prepare --recipe-path recipe.json

# Frontend builder
FROM --platform=$BUILDPLATFORM faldez/tanoshi-builder:latest AS web-builder

COPY --from=web-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY crates/tanoshi-web .
RUN trunk build --release

# Backend planner
FROM faldez/tanoshi-builder:latest AS planner

COPY crates/tanoshi crates/tanoshi
COPY crates/tanoshi-vm crates/tanoshi-vm
COPY crates/tanoshi-lib crates/tanoshi-lib
COPY crates/tanoshi-util crates/tanoshi-util
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN rm -rf crates/tanoshi/src-tauri
RUN sed -i -e 's/, "crates\/tanoshi\/src-tauri"//g' Cargo.toml

RUN cargo chef prepare --recipe-path recipe.json

# Backend builder
FROM faldez/tanoshi-builder:latest AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY crates/tanoshi crates/tanoshi
COPY crates/tanoshi-vm crates/tanoshi-vm
COPY crates/tanoshi-lib crates/tanoshi-lib
COPY crates/tanoshi-util crates/tanoshi-util
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN rm -rf crates/tanoshi/src-tauri
RUN sed -i -e 's/, "crates\/tanoshi\/src-tauri"//g' Cargo.toml

RUN cargo new crates/tanoshi-web
COPY --from=web-builder /app/dist crates/tanoshi-web/dist

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
