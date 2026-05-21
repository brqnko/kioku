FROM debian:bookworm-slim

ARG TARGETARCH

# install base tools and node
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates curl gnupg && \
    curl -fsSL https://deb.nodesource.com/setup_24.x | bash - && \
    apt-get install -y --no-install-recommends \
        sudo wget bash unzip jq build-essential nginx openssl \
        git openssh-client pkg-config libssl-dev ffmpeg \
        nodejs && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV RUST_BACKTRACE=1

# install mold
RUN MOLD_VERSION=2.41.0 && \
    ARCH=$(uname -m) && \
    curl -fsSL "https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-${ARCH}-linux.tar.gz" | \
    tar -xz -C /usr/local --strip-components=1

# configure cargo to use mold
RUN mkdir -p /root/.cargo && \
    printf '[build]\nrustflags = ["-C", "link-arg=-fuse-ld=mold"]\n' > /root/.cargo/config.toml

# install ONNX Runtime
RUN ORT_VERSION=1.20.1 && \
    ARCH=$(uname -m) && \
    ORT_ARCH=$([ "$ARCH" = "aarch64" ] && echo "aarch64" || echo "x64") && \
    curl -fsSL "https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/onnxruntime-linux-${ORT_ARCH}-${ORT_VERSION}.tgz" | \
    tar -xz -C /opt/ && \
    mv /opt/onnxruntime-linux-${ORT_ARCH}-${ORT_VERSION} /opt/onnxruntime

ENV ORT_DYLIB_PATH=/opt/onnxruntime/lib/libonnxruntime.so
ENV LD_LIBRARY_PATH=/opt/onnxruntime/lib

# model cache directory
RUN mkdir -p /opt/huggingface

# install sqlx-cli
RUN cargo install sqlx-cli --locked --no-default-features --features mysql,rustls

# install tbls
RUN wget -O /tmp/tbls.tar.gz \
        https://github.com/k1LoW/tbls/releases/download/v1.93.0/tbls_v1.93.0_linux_${TARGETARCH}.tar.gz \
    && tar -xzf /tmp/tbls.tar.gz -C /usr/local/bin tbls \
    && chmod +x /usr/local/bin/tbls \
    && rm /tmp/tbls.tar.gz

CMD sleep infinity
