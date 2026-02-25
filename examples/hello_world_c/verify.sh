#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
APP="$SCRIPT_DIR/app/main"

echo "=== 1. Generate C header ==="
cargo test -p hello_world_c -- generate_bindings --quiet

echo "=== 2. Build Rust shared library ==="
cargo build -p hello_world_c

echo "=== 3. Compile C application ==="
if [ "$(uname)" = "Darwin" ]; then
    gcc -Wall -Wextra -pedantic -std=c11 -o "$APP" "$SCRIPT_DIR/app/main.c"
else
    gcc -Wall -Wextra -pedantic -std=c11 -o "$APP" "$SCRIPT_DIR/app/main.c" -ldl
fi

echo "=== 4. Run ==="
if [ "$(uname)" = "Darwin" ]; then
    DYLD_LIBRARY_PATH="$REPO_ROOT/target/debug" "$APP"
else
    LD_LIBRARY_PATH="$REPO_ROOT/target/debug" "$APP"
fi

rm -f "$APP"
echo "=== OK ==="
