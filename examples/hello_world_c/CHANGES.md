# C Backend: Typed Enum Support, Dispatch Table, and Hello World Example

## Overview

Three features were added to the Interoptopus C backend:

1. **Typed enum support** -- Rust enums with data payloads now generate correct C11 tagged unions.
2. **Function dispatch table + loader** -- An optional loader that resolves symbols from a shared library at runtime via `dlopen`/`LoadLibrary`.
3. **`hello_world_c` example** -- A complete cross-platform end-to-end sample demonstrating both features.

---

## 1. Typed Enum Support (Tagged Unions)

### Problem

The C backend could not generate bindings for Rust enums carrying data. Every typed variant was emitted as:

```c
// TODO - OMITTED DATA VARIANT - BINDINGS ARE BROKEN
```

This affected direct enums like `EnumPayload { A, B(Vec3f32), C(u32) }` and pattern types like `Option<T>` and `Result<T, E>`, which are internally represented as enums with typed variants.

### Solution

The backend now detects whether an enum contains any `VariantKind::Typed` variants:

- **Unit-only enums** generate a plain `typedef enum` (unchanged).
- **Enums with typed variants** generate a tag enum + struct with an anonymous union (C11).

For example, this Rust enum:

```rust
#[ffi_type]
pub enum Shape {
    Circle(f32),
    Rectangle(Vec2),
}
```

Now generates:

```c
typedef enum SHAPE_TAG
    {
    SHAPE_CIRCLE = 0,
    SHAPE_RECTANGLE = 1,
    } SHAPE_TAG;

typedef struct SHAPE
    {
    SHAPE_TAG tag;
    union
        {
        float circle;
        VEC2 rectangle;
        };
    } SHAPE;
```

Key design decisions:

- The typedef name stays `SHAPE` (a struct now, not an enum), so `to_type_specifier()` needs no changes and function signatures remain stable.
- The tag enum gets a `_TAG` suffix.
- An anonymous union (C11) is used so fields are accessed as `val.circle`, not `val.data.circle`.
- Union members are named after the variant in lowercase.
- Unit variants only appear in the tag enum, not in the union.

### Impact on Option/Result

`Option<T>` and `Result<T, E>` patterns call the same `write_type_definition_enum()` path. They are now automatically converted to tagged unions too. For example, `Option<Inner>` becomes:

```c
typedef enum OPTIONINNER_TAG
    {
    OPTIONINNER_SOME = 0,
    OPTIONINNER_NONE = 1,
    } OPTIONINNER_TAG;

typedef struct OPTIONINNER
    {
    OPTIONINNER_TAG tag;
    union
        {
        INNER some;
        };
    } OPTIONINNER;
```

### Files changed

| File | Change |
|------|--------|
| `crates/backend_c/src/converters.rs` | Added `enum_to_tag_typename()` |
| `crates/backend_c/src/interop/types.rs` | Added `enum_has_typed_variants()`, `write_type_definition_enum_simple()`, `write_type_definition_enum_tagged_union()`, `write_braced_declaration_closing_anonymous()`; refactored `write_type_definition_enum()` to dispatch; fixed `write_type_definition_enum_variant()` to emit discriminants for both unit and typed variants |

---

## 2. Function Dispatch Table + Loader

### Problem

The C backend only generates function **declarations**. There was no mechanism to actually call into a Rust shared library at runtime from C without writing manual `dlopen`/`dlsym` boilerplate.

### Solution

A new optional `loader` field on the `Interop` builder. When set, the backend additionally generates:

1. A **dispatch table struct** (`interop_api_t`) containing a function pointer field for every FFI function.
2. A **cross-platform loader function** (`interop_load`) that uses `dlopen` on POSIX or `LoadLibrary` on Windows.

The POSIX loader uses `memcpy` to copy the `void*` returned by `dlsym` into the function pointer field, avoiding the ISO C prohibition on direct `void*`-to-function-pointer casts. This compiles cleanly under `-Wall -Wextra -pedantic -std=c11`.

Usage:

```rust
Interop::builder()
    .inventory(inventory)
    .loader("my_library".to_string())  // enables dispatch table + loader
    .build()?
    .write_file("bindings/my_library.h")?;
```

Generated output (excerpt):

```c
typedef struct interop_api_t
    {
        float (*shape_area)(SHAPE);
        VECDRAWCOMMAND (*create_default_commands)();
    } interop_api_t;

#if defined(_WIN32)
#include <windows.h>
static int interop_load(const char* path, interop_api_t* api)
    {
        HMODULE lib = LoadLibraryA(path);
        if (!lib) return -1;
        api->shape_area = (float (*)(SHAPE))(void*)GetProcAddress(lib, "shape_area");
        /* ... */
        return 0;
    }
#else
#include <dlfcn.h>
#include <string.h>
static int interop_load(const char* path, interop_api_t* api)
    {
        void* lib = dlopen(path, RTLD_NOW);
        if (!lib) return -1;
        void* sym;
        sym = dlsym(lib, "shape_area"); memcpy(&api->shape_area, &sym, sizeof(sym));
        /* ... */
        return 0;
    }
#endif
```

When `loader` is not set (the default), the output is unchanged -- no dispatch table or loader is emitted.

### Files changed

| File | Change |
|------|--------|
| `crates/backend_c/src/interop.rs` | Added `loader: Option<String>` field; conditionally calls `write_dispatch_table()` and `write_loader()` in `write_to()` |
| `crates/backend_c/src/interop/functions.rs` | Added `function_pointer_params_string()`, `write_dispatch_table()`, `write_loader()` |

---

## 3. `hello_world_c` Example

A new `examples/hello_world_c/` directory demonstrates both features end-to-end.

### Structure

```
examples/hello_world_c/
    Cargo.toml          # cdylib + rlib, depends on interoptopus + interoptopus_backend_c
    src/lib.rs          # Rust FFI types and functions
    bindings/
        hello_world.h   # Generated C header (tagged unions + dispatch table + loader)
    app/
        main.c          # Cross-platform C program that loads the library and calls FFI functions
```

### Cross-platform support

`main.c` uses preprocessor defines to select the correct shared library filename:

- Linux: `libhello_world_c.so`
- macOS: `libhello_world_c.dylib`
- Windows: `hello_world_c.dll`

The library is loaded by filename only (no path), so the system library search path must include the directory containing the built shared library (`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS, `PATH` on Windows).

### What the example demonstrates

- **Tagged union**: `Shape` enum with `Circle(f32)` and `Rectangle(Vec2)` variants
- **Struct containing a tagged union**: `DrawCommand { shape: Shape, position: Vec2 }`
- **Slice and Vec patterns**: `total_area(Slice<DrawCommand>)`, `create_default_commands() -> Vec<DrawCommand>`
- **Dispatch table + loader**: `main.c` loads the Rust shared library at runtime via `interop_load()`

### Files added

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root updated to include the new example |
| `examples/hello_world_c/Cargo.toml` | Crate manifest |
| `examples/hello_world_c/src/lib.rs` | Rust FFI definitions + binding generation test |
| `examples/hello_world_c/bindings/hello_world.h` | Generated C header |
| `examples/hello_world_c/app/main.c` | Cross-platform C consumer program |

---

## 4. Test & Reference File Updates

All existing reference files were regenerated via `INTEROPTOPUS_UPDATE_BINDINGS=1 cargo test`. The changes in the reference output are:

- Every `// TODO - OMITTED DATA VARIANT - BINDINGS ARE BROKEN` line is gone.
- Typed enums, `Option<T>`, and `Result<T, E>` now emit tag enum + struct with anonymous union.
- All function signatures remain unchanged (type names are stable).
- `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` all pass cleanly.
