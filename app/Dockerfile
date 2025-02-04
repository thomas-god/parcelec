# Build layer
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml /app
COPY src/ /app/src

RUN cargo build --release

# Prod layer
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/parcelec-app /

CMD ["./web"]

# Sleep for debugging
# CMD ["tail", "-f", "/dev/null"]
