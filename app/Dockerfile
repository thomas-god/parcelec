# Build layer
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml /app
COPY src/ /app/src

RUN cargo build --release

# Prod layer
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/parcelec-app /
ENV ALLOW_ORIGIN="https://parcelec.org"

CMD ["./parcelec-app"]

# Sleep for debugging
# CMD ["tail", "-f", "/dev/null"]
