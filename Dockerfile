# Build stage
FROM rust:1.83.0-slim-bullseye AS builder

# Add build arguments
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG TARGETOS
ARG TARGETARCH

# Install common build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install architecture-specific dependencies and ONNX Runtime
RUN case "$TARGETARCH" in \
        "amd64") \
            curl -L https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz | tar -xz -C /usr/local \
            ;; \
        "arm64") \
            curl -L https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-aarch64-1.16.0.tgz | tar -xz -C /usr/local \
            ;; \
    esac

WORKDIR /usr/src/app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release || true

# Copy real source
COPY src/ src/

# Build application
RUN cargo build --release && \
    cp target/release/rmbg /usr/src/app/rmbg

# Runtime stage
FROM debian:bullseye-slim

ARG TARGETARCH

RUN apt-get update && apt-get install -y \
    libgomp1 \
    ca-certificates \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create directories with appropriate permissions
RUN mkdir -p /app/tmp /app/models && \
    chmod 777 /app/tmp /app/models

# Copy ONNX Runtime based on architecture
COPY --from=builder /usr/local/onnxruntime-linux-* /usr/local/onnxruntime-linux-${TARGETARCH}-1.16.0
ENV LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-${TARGETARCH}-1.16.0/lib

# Copy binary
COPY --from=builder /usr/src/app/rmbg /app/
RUN chmod +x /app/rmbg

ENV BIND=0.0.0.0:8080 \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    UPLOAD_DIR=/app/tmp \
    MODEL_PATH=/app/models/medium.onnx \
    MODEL_SIZE=medium \
    LOG_LEVEL=debug

EXPOSE 8080

VOLUME ["/app/models", "/app/tmp"]
CMD ["./rmbg"]
