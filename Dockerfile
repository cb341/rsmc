FROM rust:1.83.0-slim-bullseye as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets

RUN apt-get update && \
    apt-get install -y pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev && \
    rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/client /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/server /usr/local/bin/
COPY --from=builder /usr/src/app/assets /usr/local/bin/assets

RUN apt-get update && \
    apt-get install -y libasound2 libudev1 && \
    rm -rf /var/lib/apt/lists/*

CMD ["server"] 