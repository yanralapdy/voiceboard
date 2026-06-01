# syntax=docker/dockerfile:1
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake clang libclang-dev libasound2-dev libpulse-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/voiceboard /voiceboard
ENTRYPOINT ["/voiceboard"]
