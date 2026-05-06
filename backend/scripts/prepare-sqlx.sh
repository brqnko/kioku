#!/bin/bash
set -euo pipefail

BACKEND_DIR="${BACKEND_DIR:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$BACKEND_DIR"

export DATABASE_URL="${DATABASE_URL:-mysql://kioku:kioku@db:3306/kioku}"

cargo sqlx prepare
