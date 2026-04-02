#!/usr/bin/env bash
set -euo pipefail
echo "=== fmt ==="
cargo fmt --check
echo "=== clippy ==="
cargo clippy --all-targets -- -D warnings
echo "=== test ==="
cargo test
echo "=== All checks passed ==="
