# syntax=docker/dockerfile:1

# Build stage
FROM --platform=$BUILDPLATFORM rust:1.83.0-slim-bullseye AS builder

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETOS
ARG TARGETARCH

RUN echo "Building on $BUILDPLATFORM for $TARGETPLATFORM ($TARGETOS/$TARGETARCH)"

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    cmake \
    gcc \
    g++ \
    git \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Install ONNX Runtime based on architecture
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        curl -L https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz | tar -xz -C /usr/local && \
        echo "export LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-x64-1.16.0/lib" >> /etc/profile; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        curl -L https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-aarch64-1.16.0.tgz | tar -xz -C /usr/local && \
        echo "export LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-aarch64-1.16.0/lib" >> /etc/profile; \
    fi

WORKDIR /usr/src/app

# Set ORT_STRATEGY to download
ENV ORT_STRATEGY=download
ENV ORT_LIB_LOCATION=/usr/local/onnxruntime-linux-${TARGETARCH}-1.16.0/lib

# Copy only Cargo.toml first
COPY Cargo.toml ./

# Create a dummy build to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"dummy\");}" > src/main.rs && \
    # Skip building ort during dependency caching
    cargo build --release || true && \
    rm -rf src

# Now copy the real source code
COPY src/ src/
COPY Cargo.lock ./

# Clean old artifacts and build
RUN cargo clean && \
    RUSTFLAGS="-C target-cpu=native" cargo build --release

# Runtime stage
FROM --platform=$TARGETPLATFORM debian:bullseye-slim

ARG TARGETARCH

RUN apt-get update && apt-get install -y \
    libgomp1 \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy ONNX Runtime based on architecture
COPY --from=builder /usr/local/onnxruntime-linux-*-1.16.0 /usr/local/onnxruntime-linux-${TARGETARCH}-1.16.0
ENV LD_LIBRARY_PATH=/usr/local/onnxruntime-linux-${TARGETARCH}-1.16.0/lib

WORKDIR /app
RUN mkdir -p /app/tmp /app/models

COPY --from=builder /usr/src/app/target/release/rmbg /app/

ENV BIND=0.0.0.0:8080 \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    UPLOAD_DIR=/app/tmp \
    MODEL_PATH=/app/models/medium.onnx \
    MODEL_SIZE=medium \
    LOG_LEVEL=debug

EXPOSE 8080

VOLUME /app/models
CMD ["./rmbg"]
