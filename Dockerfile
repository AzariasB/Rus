# Builder phase
FROM rust:1.65 as builder
WORKDIR /home/rus

COPY ./src src
COPY ./api api
COPY ./core core
COPY ./entity entity
COPY ./migration migration

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo install --path .

# Bundle phase
FROM debian:11-slim

COPY --from=builder /usr/local/cargo/bin/rus /home/rus/rus
COPY ./api/static /home/rus/api/static
COPY ./api/templates /home/rus/api/templates

WORKDIR /home/rus
CMD ["./rus"]