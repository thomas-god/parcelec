# Build layer
FROM rust:latest AS base
RUN cargo install cargo-chef

FROM base AS planner
WORKDIR /app

COPY Cargo.toml /app
COPY src/ /app/src

RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml /app
COPY src/ /app/src

RUN cargo build --release

# Prod layer
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/parcelec-app /
ENV DOMAIN=".parcelec.org"
ENV ALLOW_ORIGIN="https://parcelec.org"

CMD ["./parcelec-app"]

# Sleep for debugging
# CMD ["tail", "-f", "/dev/null"]
