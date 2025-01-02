#!/bin/bash

# Create models folder
mkdir -p 'models'

# Download small model
curl -L 'https://github.com/imgly/background-removal-js/raw/4306d99530d3ae9ec11a892a23802be28f367518/bundle/models/small' -o 'models/small.onnx'

# Download medium model
curl -L 'https://github.com/imgly/background-removal-js/raw/4306d99530d3ae9ec11a892a23802be28f367518/bundle/models/medium' -o 'models/medium.onnx'

# Download large model
curl -L 'https://github.com/imgly/background-removal-js/raw/4306d99530d3ae9ec11a892a23802be28f367518/bundle/models/large' -o 'models/large.onnx'
