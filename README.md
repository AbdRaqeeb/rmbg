# Rmbg

A Rust-based web service for removing backgrounds from images using ONNX models, built with Actix-web. This project is a rewrite of the logic from [rust-background-removal](https://github.com/dnanhkhoa/rust-background-removal) adapted to work as a web service with Actix.

## Features

- Background removal using ONNX models
- Support for multiple model sizes (small, medium, large)
- CUDA acceleration support
- Auto-cropping option
- Cloudinary integration for image storage
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
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
LOG_LEVEL=info
UPLOAD_DIR=./tmp

# Cloudinary configuration
CLOUDINARY_CLOUD_NAME=your_cloud_name
CLOUDINARY_API_KEY=your_api_key
CLOUDINARY_API_SECRET=your_api_secret
CLOUDINARY_UPLOAD_PRESET=your_upload_preset

# Model configuration
# Options: small, medium, large
MODEL_SIZE=medium  
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
POST /api/process?crop=true
Content-Type: multipart/form-data

Parameters:
- files: Array of image files
- crop: (Optional) Boolean query parameter to enable auto-cropping

Response:
{
    "results": [
        {
            "secure_url": "https://cloudinary.com/..."
        }
    ]
}
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
- Cloudinary upload issues
- Internal server errors

## Credits

This project is based on the excellent work done in [rust-background-removal](https://github.com/dnanhkhoa/rust-background-removal) by dnanhkhoa. The core background removal logic has been adapted and modified to work as a web service using Actix.
