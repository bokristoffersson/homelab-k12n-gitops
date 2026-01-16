#!/bin/bash
# Convert a HuggingFace model to MLX format (4-bit)

set -e

MODELS_DIR="./models"
Q_BITS="4"

usage() {
  cat <<'EOF'
Usage:
  ./convert_13b.sh --hf <huggingface/model> [--name <output-name>] [--q-bits 4]

Examples:
  ./convert_13b.sh --hf mistralai/Mistral-7B-Instruct-v0.3 --name Mistral-7B-Instruct-v0.3-4bit-mlx
  ./convert_13b.sh --hf mistralai/Ministral-3-3B-Instruct-2512

Notes:
  - Output is written to ./models/<name>
  - Default name is derived from HF model + q-bits
EOF
}

HF_MODEL=""
MODEL_NAME=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --hf)
      HF_MODEL="$2"
      shift 2
      ;;
    --name)
      MODEL_NAME="$2"
      shift 2
      ;;
    --q-bits)
      Q_BITS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$HF_MODEL" ]]; then
  echo "Missing --hf <huggingface/model>"
  usage
  exit 1
fi

if [[ -z "$MODEL_NAME" ]]; then
  MODEL_NAME="$(basename "$HF_MODEL")-${Q_BITS}bit-mlx"
fi

echo "Converting ${HF_MODEL} to MLX format..."
echo "Output: ${MODELS_DIR}/${MODEL_NAME}"
echo "Quantization: ${Q_BITS}-bit"

# Create models directory if it doesn't exist
mkdir -p "${MODELS_DIR}"

# Use venv python if available, otherwise use system python
if [ -f "./venv/bin/python3" ]; then
    PYTHON="./venv/bin/python3"
else
    PYTHON="python3"
fi

# Convert the model
"${PYTHON}" -m mlx_lm convert \
  --hf-path "${HF_MODEL}" \
  --mlx-path "${MODELS_DIR}/${MODEL_NAME}" \
  --quantize \
  --q-bits "${Q_BITS}"

echo "Conversion complete!"
echo "Model saved to: ${MODELS_DIR}/${MODEL_NAME}"
