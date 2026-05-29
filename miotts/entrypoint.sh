#!/bin/bash
set -e

: "${MODEL_REPO:=Aratako/MioTTS-GGUF}"
: "${MODEL_FILE:=MioTTS-0.6B-Q8_0.gguf}"
: "${LLM_PORT:=8000}"
: "${TTS_PORT:=8001}"
: "${DEVICE:=cpu}"

llama-server \
  -hf "${MODEL_REPO}" -hff "${MODEL_FILE}" \
  -c 8192 --host 0.0.0.0 --port "${LLM_PORT}" \
  &

echo "Waiting for LLM server on port ${LLM_PORT}..."
until curl -sf "http://localhost:${LLM_PORT}/health" >/dev/null 2>&1; do
  sleep 1
done
echo "LLM server ready."

exec uv run python run_server.py \
  --host 0.0.0.0 \
  --port "${TTS_PORT}" \
  --llm-base-url "http://localhost:${LLM_PORT}/v1" \
  --device "${DEVICE}" \
  "$@"
