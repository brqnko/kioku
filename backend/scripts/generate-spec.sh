#!/bin/bash
set -euo pipefail

OUTPUT_DIR="../shared/api"

mkdir -p "$OUTPUT_DIR"
cargo run --bin gen_spec > "$OUTPUT_DIR/openapi.yaml"
