#!/bin/bash
set -e

: "${MODEL_REPO:=Aratako/MioTTS-GGUF}"
: "${MODEL_FILE:=MioTTS-0.6B-Q8_0.gguf}"
: "${LLM_PORT:=8000}"
: "${TTS_PORT:=8001}"
: "${DEVICE:=cpu}"
# In a container, nproc/std::thread::hardware_concurrency report the HOST core
# count, not the cgroup CPU quota. Without an explicit thread count llama-server
# oversubscribes the allocated vCPUs and inference crawls (~1 t/s). Pin threads
# to the allocated CPUs via LLAMA_THREADS (set it to the container's vCPU count).
: "${LLAMA_THREADS:=$(nproc)}"
# Keep the PyTorch/MioCodec side from oversubscribing as well.
export OMP_NUM_THREADS="${OMP_NUM_THREADS:-${LLAMA_THREADS}}"

llama-server \
  -hf "${MODEL_REPO}" -hff "${MODEL_FILE}" \
  -c 8192 --host 0.0.0.0 --port "${LLM_PORT}" \
  --threads "${LLAMA_THREADS}" --threads-batch "${LLAMA_THREADS}" \
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
