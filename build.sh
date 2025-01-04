#!/bin/bash

# Default values
TAG="latest"
REGISTRY="ghcr.io/abdraqeeb"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --tag) TAG="$2"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

echo "Building images with tag: $TAG"

# Create and use a new builder instance with multi-architecture support
docker buildx create --name multiarch-builder --driver docker-container --bootstrap || true
docker buildx use multiarch-builder

# Build and push multi-architecture image
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag $REGISTRY/rmbg:"$TAG" \
  --tag $REGISTRY/rmbg:latest \
  --push \
  .

# Clean up
docker buildx rm multiarch-builder
