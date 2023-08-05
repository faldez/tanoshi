# Frontend planner
FROM faldez/tanoshi-builder:latest AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# Backend builder
FROM faldez/tanoshi-builder:latest AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cd crates/tanoshi-web && trunk build --release

RUN cargo build -p tanoshi --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/tanoshi .
RUN chmod +x tanoshi

RUN apt update && apt upgrade -y && apt install --reinstall -y ca-certificates libssl3

ENV PORT=80
ENV TANOSHI_LOG=info
ENV TANOSHI_HOME=/tanoshi

EXPOSE $PORT

ENTRYPOINT ["/app/tanoshi"]
