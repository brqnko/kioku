# GPU build of Irodori-TTS-Server (OpenAI-compatible /v1/audio/speech) for Azure
# Container Apps NC8as-T4 (NVIDIA T4). The cu128 torch wheel bundles CUDA, so a
# plain python base works on the GPU node (the node provides the driver).
FROM python:3.10-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        git cmake build-essential ffmpeg libsndfile1 curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/

RUN git clone https://github.com/Aratako/Irodori-TTS-Server.git /app
WORKDIR /app
# Install the CUDA 12.8 backend (bundled CUDA torch).
RUN uv sync --locked --no-dev --extra cu128

ENV IRODORI_HOST=0.0.0.0 \
    IRODORI_PORT=8088 \
    IRODORI_HF_CHECKPOINT=Aratako/Irodori-TTS-500M-v3 \
    IRODORI_CODEC_REPO=Aratako/Semantic-DACVAE-Japanese-32dim \
    IRODORI_MODEL_DEVICE=cuda \
    IRODORI_CODEC_DEVICE=cuda \
    IRODORI_MODEL_PRECISION=bf16 \
    IRODORI_CODEC_PRECISION=bf16 \
    IRODORI_PRELOAD=true \
    IRODORI_ALLOW_NO_REF_VOICE=true \
    HF_HOME=/root/.cache/huggingface

EXPOSE 8088
CMD ["/app/.venv/bin/python", "-m", "irodori_openai_tts"]
