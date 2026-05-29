# ── Stage 1: Build llama.cpp server ───────────────────────────────────────────
FROM debian:bookworm-slim AS llama-build

RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake build-essential git libcurl4-openssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 https://github.com/ggml-org/llama.cpp.git /src
WORKDIR /src
RUN cmake -B build -DLLAMA_CURL=ON -DBUILD_SHARED_LIBS=OFF \
    && cmake --build build --target llama-server -j"$(nproc)"

# ── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM python:3.12-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
        git cmake build-essential libsndfile1 libcurl4 curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=llama-build /src/build/bin/llama-server /usr/local/bin/
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/

RUN git clone https://github.com/Aratako/MioTTS-Inference.git /app
WORKDIR /app
RUN uv sync --frozen

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENV MODEL_REPO=Aratako/MioTTS-GGUF \
    MODEL_FILE=MioTTS-0.6B-Q8_0.gguf \
    LLM_PORT=8000 \
    TTS_PORT=8001 \
    DEVICE=cpu

EXPOSE 8001
ENTRYPOINT ["/entrypoint.sh"]
