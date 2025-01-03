# Background Removal API

A Rust-based web service for removing backgrounds from images using ONNX models, built with Actix-web. This project is a rewrite of the logic from [rust-background-removal](https://github.com/dnanhkhoa/rust-background-removal) adapted to work as a web service with Actix.

## Features

- Background removal using ONNX models
- Support for multiple model sizes (small, medium, large)
- Multiple storage provider support:
    - Cloudinary (default)
    - AWS S3
    - MinIO
- CUDA acceleration support
- Auto-cropping option
- Concurrent processing of multiple images
- Health check endpoint

## Prerequisites

- Rust (latest stable version)
- CUDA toolkit (optional, for GPU acceleration)
- CoreML (optional, for MacOS acceleration)

## Setup

1. Clone the repository:
```bash
git clone https://github.com/AbdRaqeeb/rmbg
cd rmbg
```

2. Download the ONNX models by running the provided script:
```bash
chmod +x download_models.sh
./download_models.sh
```

3. Create a `.env` file in the project root with the following variables:
```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
LOG_LEVEL=info
UPLOAD_DIR=./tmp

# Cloudinary configuration
CLOUDINARY_CLOUD_NAME=your_cloud_name
CLOUDINARY_API_KEY=your_api_key
CLOUDINARY_API_SECRET=your_api_secret
CLOUDINARY_UPLOAD_PRESET=your_upload_preset

# AWS S3 Configuration
AWS_ACCESS_KEY_ID=your_aws_access_key
AWS_SECRET_ACCESS_KEY=your_aws_secret_key
AWS_REGION=us-east-1
S3_BUCKET=your-bucket-name

# MinIO Configuration
MINIO_ACCESS_KEY=your_minio_access_key
MINIO_SECRET_KEY=your_minio_secret_key
MINIO_BUCKET=your-minio-bucket
MINIO_ENDPOINT=localhost:9000
MINIO_SECURE=false  # Use true for HTTPS

# Model configuration
MODEL_SIZE=medium  # Options: small, medium, large
MODEL_PATH=models/medium.onnx
```

4. Install dependencies and build the project:
```bash
cargo build --release
```

## Running the Service

1. Start the server:
```bash
cargo run --release
```

2. The API will be available at `http://localhost:8080` (or whatever host/port you configured)

## API Endpoints

### Health Check
```
GET /
Response: "healthy"
```

### Process Images
```
POST /api/process
Content-Type: multipart/form-data

Query Parameters:
- upload: Storage provider to use (cloudinary, s3, minio)
- crop: Boolean flag for auto-cropping (optional)

Parameters:
- files: Array of image files

Response:
{
    "results": [
        {
            "secure_url": "https://storage-provider.com/path/to/image.png"
        }
    ]
}
```

## Usage Examples

### Using Different Storage Providers

1. Cloudinary (default):
```bash
curl -X POST "http://localhost:8080/api/process" \
  -F "files=@image.jpg"
```

2. AWS S3:
```bash
curl -X POST "http://localhost:8080/api/process?upload=s3" \
  -F "files=@image.jpg"
```

3. MinIO:
```bash
curl -X POST "http://localhost:8080/api/process?upload=minio" \
  -F "files=@image.jpg"
```

4. With Auto-cropping:
```bash
curl -X POST "http://localhost:8080/api/process?crop=true" \
  -F "files=@image.jpg"
```

## Performance Optimization

The service supports multiple execution providers:
1. CUDA (GPU acceleration)
2. CoreML (MacOS acceleration)
3. CPU (fallback)

The execution providers are attempted in order, using the first available one.

## Error Handling

The service provides detailed error responses for various scenarios:
- Invalid file format
- Image processing failures
- Storage provider upload issues
- Internal server errors

## Credits

This project is based on the excellent work done in [rust-background-removal](https://github.com/dnanhkhoa/rust-background-removal) by dnanhkhoa. The core background removal logic has been adapted and modified to work as a web service using Actix.
