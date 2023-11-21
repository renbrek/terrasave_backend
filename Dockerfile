# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.73.0
ARG APP_NAME=terrasave_backend
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update
RUN apt-get install -y pkg-config openssl libssl-dev curl

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM debian:bullseye-slim AS final

WORKDIR /app
COPY --from=build /bin/server server

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

RUN chown -R appuser:appuser /app
RUN chmod -R 755 /app

USER appuser

EXPOSE 3000

CMD ["/app/server"]
