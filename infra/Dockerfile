# syntax=docker/dockerfile:1

# --- Client: build the SvelteKit static site ---
FROM node:24 AS client-builder
WORKDIR /client

COPY client/package.json client/package-lock.json ./
RUN npm ci

COPY client/vite.config.ts ./
COPY client/tsconfig.json ./
COPY client/.npmrc ./
COPY client/.env.production ./.env
COPY client/src ./src
COPY client/public ./public
COPY client/static ./static

ENV NODE_ENV=production
RUN npm run build

# --- Server: build the Rust binary ---
FROM rust:latest AS base
RUN cargo install cargo-chef

FROM base AS planner
WORKDIR /app
COPY app/Cargo.toml app/Cargo.lock ./
COPY app/src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY app/Cargo.toml app/Cargo.lock ./
COPY app/src ./src
RUN cargo build --release

# --- Runtime: single image serving both API and static client ---
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/parcelec-app /
COPY --from=client-builder /client/build /public
ENV ALLOW_ORIGIN="https://parcelec.org"
ENV STATIC_DIR="/public"
CMD ["./parcelec-app"]
