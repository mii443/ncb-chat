FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && apt-get install -y --no-install-recommends libssl-dev pkg-config gcc && apt-get -y clean
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /ncb-ping
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl-dev && apt-get -y clean
COPY --from=builder /app/target/release/ncb-ping /usr/local/bin
ENTRYPOINT ["/usr/local/bin/ncb-ping"]
