#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"

echo "=== Configure & build C++ tests (hello_world) ==="
cmake -S "$SCRIPT_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Debug 2>&1
cmake --build "$BUILD_DIR" 2>&1

echo "=== Run tests ==="
if [ "$(uname)" = "Darwin" ]; then
    DYLD_LIBRARY_PATH="$REPO_ROOT/target/debug" ctest --test-dir "$BUILD_DIR" --output-on-failure
else
    LD_LIBRARY_PATH="$REPO_ROOT/target/debug" ctest --test-dir "$BUILD_DIR" --output-on-failure
fi

echo "=== OK ==="
