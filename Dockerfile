FROM rust:1-slim AS build
RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/* \
    && curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
WORKDIR /app
COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN wasm-pack build --target web --features wasm && cp pkg/* www/pkg/
RUN cargo build --release -p scrib-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/scrib-server /usr/local/bin/scrib-server
COPY --from=build /app/www /app/www
WORKDIR /app
ENV PORT=10000
EXPOSE 10000
CMD ["scrib-server", "--addr", "0.0.0.0"]
