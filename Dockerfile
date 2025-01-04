FROM rust:1.83.0-slim-bullseye AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install ONNX Runtime
RUN curl -L https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz | tar -xz -C /usr/local
ENV LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-x64-1.16.0/lib

WORKDIR /usr/src/app

# Build dependencies
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Build application
COPY src src/
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libgomp1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy ONNX Runtime
COPY --from=builder /usr/local/onnxruntime-linux-x64-1.16.0 /usr/local/onnxruntime-linux-x64-1.16.0
ENV LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-x64-1.16.0/lib

WORKDIR /app
RUN mkdir -p /app/tmp /app/models

COPY --from=builder /usr/src/app/target/release/rmbg /app/

ENV BIND=0.0.0.0:8080 \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    UPLOAD_DIR=/app/tmp \
    MODEL_PATH=/app/models/medium.onnx \
    MODEL_SIZE=medium \
    RUST_LOG=debug

EXPOSE 8080

VOLUME /app/models
CMD ["./rmbg"]
