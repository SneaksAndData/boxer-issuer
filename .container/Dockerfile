# Specify the base image that has Rust installed
FROM rust:1.67 as build-env

WORKDIR /app/src

COPY . .

RUN cargo build --release

ARG TARGETARCH

FROM debian:bookworkworm
RUN apt-get update -y && apt-get install -y curl

WORKDIR /app
COPY --from=build-env /app/target/release .
USER app
ENTRYPOINT ["boxer-issuer"]

