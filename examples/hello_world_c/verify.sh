#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$SCRIPT_DIR/tests/build"

echo "=== 1. Generate C header ==="
cargo test -p hello_world_c -- generate_bindings --quiet

echo "=== 2. Build Rust shared library ==="
cargo build -p hello_world_c

echo "=== 3. Configure & build C++ tests ==="
cmake -S "$SCRIPT_DIR/tests" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Debug 2>&1
cmake --build "$BUILD_DIR" 2>&1

echo "=== 4. Run tests ==="
if [ "$(uname)" = "Darwin" ]; then
    DYLD_LIBRARY_PATH="$REPO_ROOT/target/debug" ctest --test-dir "$BUILD_DIR" --output-on-failure
else
    LD_LIBRARY_PATH="$REPO_ROOT/target/debug" ctest --test-dir "$BUILD_DIR" --output-on-failure
fi

echo "=== OK ==="
