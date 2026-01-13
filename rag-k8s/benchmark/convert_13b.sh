#!/bin/bash
# Convert Meta-Llama-3-13B-Instruct from HuggingFace to MLX format
# NOTE: Meta doesn't have an official Llama-3-13B model. This uses a community-merged version.
# Official Meta models: Llama-3-8B-Instruct, Llama-3-70B-Instruct

set -e

MODEL_NAME="Meta-Llama-3-13B-Instruct-4bit-mlx"
MODELS_DIR="./models"
# Using community-merged version since Meta doesn't have official 13B
HF_MODEL="andrijdavid/Meta-Llama-3-13B-Instruct"

echo "Converting ${HF_MODEL} to MLX format..."
echo "Output: ${MODELS_DIR}/${MODEL_NAME}"

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
  --q-bits 4

echo "Conversion complete!"
echo "Model saved to: ${MODELS_DIR}/${MODEL_NAME}"
