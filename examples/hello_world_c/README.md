# Hello World C Example

Demonstrates calling a Rust library from C using Interoptopus-generated bindings, including typed enums (tagged unions) and a runtime dispatch table loader.

## Prerequisites

- Rust toolchain (1.88+)
- A C compiler (`gcc` or `clang`)

## Steps

All commands are run from the repository root.

### 1. Generate the C header

```sh
cargo test -p hello_world_c
```

This runs the `generate_bindings` test inside `src/lib.rs`, which writes `examples/hello_world_c/bindings/hello_world.h`.

### 2. Build the Rust shared library

```sh
cargo build -p hello_world_c
```

This produces `target/debug/libhello_world_c.so` (Linux), `target/debug/libhello_world_c.dylib` (macOS), or `target/debug/hello_world_c.dll` (Windows).

### 3. Compile the C application

Linux:

```sh
gcc -Wall -pedantic -std=c11 -o examples/hello_world_c/app/main examples/hello_world_c/app/main.c -ldl
```

macOS (no `-ldl` needed, `dlopen` is in libSystem):

```sh
gcc -Wall -pedantic -std=c11 -o examples/hello_world_c/app/main examples/hello_world_c/app/main.c
```

Windows (MSVC):

```sh
cl /W4 examples\hello_world_c\app\main.c /Fe:examples\hello_world_c\app\main.exe
```

### 4. Run

Linux:

```sh
LD_LIBRARY_PATH=target/debug ./examples/hello_world_c/app/main
```

macOS:

```sh
DYLD_LIBRARY_PATH=target/debug ./examples/hello_world_c/app/main
```

Windows:

```sh
copy target\debug\hello_world_c.dll examples\hello_world_c\app\
examples\hello_world_c\app\main.exe
```

### Expected output

```
Circle area: 78.539818
Rectangle area: 12.000000
Total area of 2 commands: 90.539818
```

## What it shows

- **Tagged unions**: The Rust `Shape` enum (`Circle(f32)` / `Rectangle(Vec2)`) becomes a `SHAPE_TAG` enum + `SHAPE` struct with an anonymous union in C.
- **Dispatch table**: All FFI functions are collected into an `interop_api_t` struct.
- **Runtime loader**: `interop_load()` resolves every symbol from the shared library via `dlopen`/`dlsym` (POSIX) or `LoadLibrary`/`GetProcAddress` (Windows).
- **Slices and Vecs**: `total_area` accepts a `SLICEDRAWCOMMAND`; `create_default_commands` returns a `VECDRAWCOMMAND`.
