//! Minimal C header generator for [Interoptopus](https://github.com/ralfbiedert/interoptopus).
//!
//! Generates a single `.h` file from a [`RustInventory`] containing type definitions,
//! function declarations, a dispatch table struct, and a cross-platform dynamic loader.

use interoptopus::inventory::{RustInventory, TypeId};
use interoptopus::lang::function::{Function, Signature};
use interoptopus::lang::types::{Enum, Primitive, Struct, Type, TypeKind, TypePattern, VariantKind};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

/// Generate a C header file from the given inventory.
///
/// `loader_name` controls the dispatch table and loader function names
/// (e.g. `"hello_world_c"` produces `hello_world_c_api_t` and `hello_world_c_load`).
///
/// # Errors
/// Returns an error if writing the file fails.
pub fn generate(loader_name: &str, inv: &RustInventory, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = String::new();
    let ctx = Ctx { inv, loader_name };

    writeln!(out, "#ifndef interoptopus_generated")?;
    writeln!(out, "#define interoptopus_generated")?;
    writeln!(out)?;
    writeln!(out, "#ifdef __cplusplus")?;
    writeln!(out, "extern \"C\" {{")?;
    writeln!(out, "#endif")?;
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    writeln!(out, "#include <stdbool.h>")?;
    writeln!(out)?;

    // Collect and sort types topologically
    let sorted = topo_sort_types(inv);
    for tid in &sorted {
        let ty = &inv.types[tid];
        ctx.write_type_def(&mut out, tid, ty)?;
    }

    writeln!(out)?;

    // Function declarations
    let functions = sorted_functions(inv);
    for f in &functions {
        ctx.write_function_decl(&mut out, f)?;
    }

    writeln!(out)?;

    // Dispatch table
    let struct_name = format!("{loader_name}_api_t");
    writeln!(out, "typedef struct {struct_name}")?;
    writeln!(out, "{{")?;
    for f in &functions {
        let rval = ctx.type_specifier(&f.signature.rval);
        let name = &f.name;
        let params = ctx.fn_params_types_only(&f.signature);
        writeln!(out, "    {rval} (*{name})({params});")?;
    }
    writeln!(out, "}} {struct_name};")?;
    writeln!(out)?;

    // Dynamic loader
    ctx.write_loader(&mut out, &functions)?;

    // Static loader
    ctx.write_static_loader(&mut out, &functions)?;

    writeln!(out, "#ifdef __cplusplus")?;
    writeln!(out, "}}")?;
    writeln!(out, "#endif")?;
    writeln!(out)?;
    writeln!(out, "#endif /* interoptopus_generated */")?;

    fs::write(path, out)?;
    Ok(())
}

struct Ctx<'a> {
    inv: &'a RustInventory,
    loader_name: &'a str,
}

impl Ctx<'_> {
    fn resolve(&self, tid: &TypeId) -> &Type {
        &self.inv.types[tid]
    }

    fn type_name(&self, tid: &TypeId) -> String {
        let ty = self.resolve(tid);
        match &ty.kind {
            TypeKind::Primitive(p) => primitive_c_name(*p).to_string(),
            TypeKind::ReadPointer(inner) => format!("const {}*", self.type_name(inner)),
            TypeKind::ReadWritePointer(inner) => format!("{}*", self.type_name(inner)),
            TypeKind::TypePattern(TypePattern::CChar) => "char".to_string(),
            TypeKind::TypePattern(TypePattern::Bool) => "bool".to_string(),
            TypeKind::TypePattern(TypePattern::CVoid) => "void".to_string(),
            _ => sanitize_c_name(&ty.name),
        }
    }

    fn type_specifier(&self, tid: &TypeId) -> String {
        let ty = self.resolve(tid);
        match &ty.kind {
            TypeKind::Primitive(Primitive::Void) => "void".to_string(),
            _ => self.type_name(tid),
        }
    }

    fn fn_params_types_only(&self, sig: &Signature) -> String {
        if sig.arguments.is_empty() {
            return "void".to_string();
        }
        sig.arguments.iter().map(|a| self.type_specifier(&a.ty)).collect::<Vec<_>>().join(", ")
    }

    fn fn_params_with_names(&self, sig: &Signature) -> String {
        if sig.arguments.is_empty() {
            return "void".to_string();
        }
        sig.arguments
            .iter()
            .map(|a| {
                let ty = self.type_specifier(&a.ty);
                let name = a.name.to_uppercase();
                format!("{ty} {name}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn write_type_def(&self, out: &mut String, _tid: &TypeId, ty: &Type) -> Result<(), std::fmt::Error> {
        let name = sanitize_c_name(&ty.name);
        match &ty.kind {
            TypeKind::Primitive(_) | TypeKind::ReadPointer(_) | TypeKind::ReadWritePointer(_) => {}
            TypeKind::TypePattern(TypePattern::Bool | TypePattern::CChar | TypePattern::CVoid) => {}

            TypeKind::Struct(s) => {
                self.write_struct(out, &name, s)?;
                writeln!(out)?;
            }

            TypeKind::Enum(e) => {
                self.write_enum(out, &name, e)?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::NamedCallback(sig)) => {
                self.write_callback(out, &name, sig)?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::Slice(inner) | TypePattern::SliceMut(inner)) => {
                let inner_name = self.type_specifier(inner);
                let is_mut = matches!(ty.kind, TypeKind::TypePattern(TypePattern::SliceMut(_)));
                let const_q = if is_mut { "" } else { "const " };
                writeln!(out, "typedef struct {name}")?;
                writeln!(out, "{{")?;
                writeln!(out, "    {const_q}{inner_name}* data;")?;
                writeln!(out, "    uint64_t len;")?;
                writeln!(out, "}} {name};")?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::Vec(inner)) => {
                let inner_name = self.type_specifier(inner);
                writeln!(out, "typedef struct {name}")?;
                writeln!(out, "{{")?;
                writeln!(out, "    {inner_name}* ptr;")?;
                writeln!(out, "    uint64_t len;")?;
                writeln!(out, "    uint64_t capacity;")?;
                writeln!(out, "}} {name};")?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::Utf8String) => {
                writeln!(out, "typedef struct {name}")?;
                writeln!(out, "{{")?;
                writeln!(out, "    uint8_t* ptr;")?;
                writeln!(out, "    uint64_t len;")?;
                writeln!(out, "    uint64_t capacity;")?;
                writeln!(out, "}} {name};")?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::Option(inner)) => {
                self.write_option_enum(out, &name, inner)?;
                writeln!(out)?;
            }

            TypeKind::TypePattern(TypePattern::Result(ok, err)) => {
                self.write_result_enum(out, &name, ok, err)?;
                writeln!(out)?;
            }

            TypeKind::FnPointer(sig) => {
                let rval = self.type_specifier(&sig.rval);
                let params = self.fn_params_types_only(sig);
                writeln!(out, "typedef {rval} (*{name})({params});")?;
                writeln!(out)?;
            }

            TypeKind::Opaque | TypeKind::Service => {
                writeln!(out, "typedef struct {name} {name};")?;
                writeln!(out)?;
            }

            _ => {}
        }
        Ok(())
    }

    fn write_struct(&self, out: &mut String, name: &str, s: &Struct) -> Result<(), std::fmt::Error> {
        writeln!(out, "typedef struct {name}")?;
        writeln!(out, "{{")?;
        for f in &s.fields {
            let fty = self.resolve(&f.ty);
            if let TypeKind::Array(arr) = &fty.kind {
                let elem = self.type_specifier(&arr.ty);
                writeln!(out, "    {} {}[{}];", elem, f.name, arr.len)?;
            } else {
                let ty_name = self.type_specifier(&f.ty);
                writeln!(out, "    {} {};", ty_name, f.name)?;
            }
        }
        writeln!(out, "}} {name};")?;
        Ok(())
    }

    fn write_enum(&self, out: &mut String, name: &str, e: &Enum) -> Result<(), std::fmt::Error> {
        let has_data = e.variants.iter().any(|v| matches!(v.kind, VariantKind::Tuple(_)));

        if has_data {
            // Tagged union
            let tag_name = format!("{name}_TAG");
            writeln!(out, "typedef enum {tag_name}")?;
            writeln!(out, "{{")?;
            for (i, v) in e.variants.iter().enumerate() {
                let vname = format!("{name}_{}", v.name.to_uppercase());
                writeln!(out, "    {vname} = {i},")?;
            }
            writeln!(out, "}} {tag_name};")?;
            writeln!(out)?;
            writeln!(out, "typedef struct {name}")?;
            writeln!(out, "{{")?;
            writeln!(out, "    {tag_name} tag;")?;
            writeln!(out, "    union")?;
            writeln!(out, "    {{")?;
            for v in &e.variants {
                if let VariantKind::Tuple(inner_tid) = &v.kind {
                    let inner_ty = self.type_specifier(inner_tid);
                    let field_name = v.name.to_lowercase();
                    writeln!(out, "        {inner_ty} {field_name};")?;
                }
            }
            writeln!(out, "    }};")?;
        } else {
            // Simple enum
            writeln!(out, "typedef enum {name}")?;
            writeln!(out, "{{")?;
            for (i, v) in e.variants.iter().enumerate() {
                let vname = format!("{name}_{}", v.name.to_uppercase());
                writeln!(out, "    {vname} = {i},")?;
            }
        }
        writeln!(out, "}} {name};")?;
        Ok(())
    }

    fn write_option_enum(&self, out: &mut String, name: &str, inner: &TypeId) -> Result<(), std::fmt::Error> {
        let tag_name = format!("{name}_TAG");
        writeln!(out, "typedef enum {tag_name}")?;
        writeln!(out, "{{")?;
        writeln!(out, "    {name}_SOME = 0,")?;
        writeln!(out, "    {name}_NONE = 1,")?;
        writeln!(out, "}} {tag_name};")?;
        writeln!(out)?;

        let inner_ty = self.type_specifier(inner);
        writeln!(out, "typedef struct {name}")?;
        writeln!(out, "{{")?;
        writeln!(out, "    {tag_name} tag;")?;
        writeln!(out, "    union")?;
        writeln!(out, "    {{")?;
        writeln!(out, "        {inner_ty} some;")?;
        writeln!(out, "    }};")?;
        writeln!(out, "}} {name};")?;
        Ok(())
    }

    fn write_result_enum(&self, out: &mut String, name: &str, ok: &TypeId, err: &TypeId) -> Result<(), std::fmt::Error> {
        let tag_name = format!("{name}_TAG");
        writeln!(out, "typedef enum {tag_name}")?;
        writeln!(out, "{{")?;
        writeln!(out, "    {name}_OK = 0,")?;
        writeln!(out, "    {name}_ERR = 1,")?;
        writeln!(out, "}} {tag_name};")?;
        writeln!(out)?;

        let ok_ty = self.type_specifier(ok);
        let err_ty = self.type_specifier(err);
        writeln!(out, "typedef struct {name}")?;
        writeln!(out, "{{")?;
        writeln!(out, "    {tag_name} tag;")?;
        writeln!(out, "    union")?;
        writeln!(out, "    {{")?;
        writeln!(out, "        {ok_ty} ok;")?;
        writeln!(out, "        {err_ty} err;")?;
        writeln!(out, "    }};")?;
        writeln!(out, "}} {name};")?;
        Ok(())
    }

    fn write_callback(&self, out: &mut String, name: &str, sig: &Signature) -> Result<(), std::fmt::Error> {
        let mut param_strs: Vec<String> = sig.arguments.iter().map(|a| self.type_specifier(&a.ty)).collect();
        param_strs.push("const void*".to_string());
        let params = param_strs.join(", ");
        let rval = self.type_specifier(&sig.rval);
        let fn_name = format!("{name}_fn");
        writeln!(out, "typedef {rval} (*{fn_name})({params});")?;
        writeln!(out)?;
        writeln!(out, "typedef struct {name}")?;
        writeln!(out, "{{")?;
        writeln!(out, "    {fn_name} callback;")?;
        writeln!(out, "    const void* data;")?;
        writeln!(out, "    void (*destructor)(const void*);")?;
        writeln!(out, "}} {name};")?;
        Ok(())
    }

    fn write_function_decl(&self, out: &mut String, f: &Function) -> Result<(), std::fmt::Error> {
        let rval = self.type_specifier(&f.signature.rval);
        let params = self.fn_params_with_names(&f.signature);
        writeln!(out, "{rval} {name}({params});", name = f.name)?;
        Ok(())
    }

    fn write_loader(&self, out: &mut String, functions: &[&Function]) -> Result<(), std::fmt::Error> {
        let struct_name = format!("{}_api_t", self.loader_name);
        let fn_name = format!("{}_load", self.loader_name);

        // Windows
        writeln!(out, "#if defined(_WIN32)")?;
        writeln!(out, "#include <windows.h>")?;
        writeln!(out, "static int {fn_name}(const char* path, {struct_name}* api)")?;
        writeln!(out, "{{")?;
        writeln!(out, "    HMODULE lib = LoadLibraryA(path);")?;
        writeln!(out, "    if (!lib) return -1;")?;
        for f in functions {
            let name = &f.name;
            let rval = self.type_specifier(&f.signature.rval);
            let params = self.fn_params_types_only(&f.signature);
            writeln!(out, "    api->{name} = ({rval} (*)({params}))(void*)GetProcAddress(lib, \"{name}\");")?;
            writeln!(out, "    if (!api->{name}) return -1;")?;
        }
        writeln!(out, "    return 0;")?;
        writeln!(out, "}}")?;

        // POSIX
        writeln!(out, "#else")?;
        writeln!(out, "#include <dlfcn.h>")?;
        writeln!(out, "#include <string.h>")?;
        writeln!(out, "static int {fn_name}(const char* path, {struct_name}* api)")?;
        writeln!(out, "{{")?;
        writeln!(out, "    void* lib = dlopen(path, RTLD_NOW);")?;
        writeln!(out, "    if (!lib) return -1;")?;
        writeln!(out, "    void* sym;")?;
        for f in functions {
            let name = &f.name;
            writeln!(out, "    sym = dlsym(lib, \"{name}\");")?;
            writeln!(out, "    if (!sym) return -1;")?;
            writeln!(out, "    memcpy(&api->{name}, &sym, sizeof(sym));")?;
        }
        writeln!(out, "    return 0;")?;
        writeln!(out, "}}")?;
        writeln!(out, "#endif")?;
        writeln!(out)?;
        Ok(())
    }

    fn write_static_loader(&self, out: &mut String, functions: &[&Function]) -> Result<(), std::fmt::Error> {
        let struct_name = format!("{}_api_t", self.loader_name);
        let fn_name = format!("{}_load_static", self.loader_name);
        let guard = format!("{}_STATIC", self.loader_name.to_uppercase());

        writeln!(out, "#ifdef {guard}")?;
        writeln!(out, "static int {fn_name}({struct_name}* api)")?;
        writeln!(out, "{{")?;
        for f in functions {
            let name = &f.name;
            writeln!(out, "    api->{name} = {name};")?;
        }
        writeln!(out, "    return 0;")?;
        writeln!(out, "}}")?;
        writeln!(out, "#endif /* {guard} */")?;
        writeln!(out)?;
        Ok(())
    }
}

/// Turn a Rust type name like `Option<Vec2>` into a valid C identifier like `OPTIONVEC2`.
fn sanitize_c_name(name: &str) -> String {
    name.chars()
        .filter_map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => Some(c.to_ascii_uppercase()),
            _ => None,
        })
        .collect()
}

fn primitive_c_name(p: Primitive) -> &'static str {
    match p {
        Primitive::Void => "void",
        Primitive::Bool => "bool",
        Primitive::U8 => "uint8_t",
        Primitive::U16 => "uint16_t",
        Primitive::U32 => "uint32_t",
        Primitive::U64 => "uint64_t",
        Primitive::Usize => "size_t",
        Primitive::I8 => "int8_t",
        Primitive::I16 => "int16_t",
        Primitive::I32 => "int32_t",
        Primitive::I64 => "int64_t",
        Primitive::Isize => "ptrdiff_t",
        Primitive::F32 => "float",
        Primitive::F64 => "double",
    }
}

fn sorted_functions(inv: &RustInventory) -> Vec<&Function> {
    let mut fns: Vec<&Function> = inv.functions.values().collect();
    fns.sort_by_key(|f| &f.name);
    fns
}

/// Topologically sort types so dependencies come before dependents.
fn topo_sort_types(inv: &RustInventory) -> Vec<TypeId> {
    let mut visited = HashSet::new();
    let mut order = Vec::new();

    // Build a quick set of which TypeIds are in the inventory
    let known: HashSet<&TypeId> = inv.types.keys().collect();

    for tid in inv.types.keys() {
        topo_visit(tid, inv, &known, &mut visited, &mut order);
    }
    order
}

fn topo_visit(tid: &TypeId, inv: &RustInventory, known: &HashSet<&TypeId>, visited: &mut HashSet<TypeId>, order: &mut Vec<TypeId>) {
    if !known.contains(tid) || visited.contains(tid) {
        return;
    }
    visited.insert(*tid);

    let ty = &inv.types[tid];
    for dep in type_deps(ty) {
        topo_visit(&dep, inv, known, visited, order);
    }
    order.push(*tid);
}

fn type_deps(ty: &Type) -> Vec<TypeId> {
    let mut deps = Vec::new();
    match &ty.kind {
        TypeKind::Struct(s) => {
            for f in &s.fields {
                deps.push(f.ty);
            }
        }
        TypeKind::Enum(e) => {
            for v in &e.variants {
                if let VariantKind::Tuple(tid) = &v.kind {
                    deps.push(*tid);
                }
            }
        }
        TypeKind::TypePattern(TypePattern::Slice(t) | TypePattern::SliceMut(t) | TypePattern::Vec(t) | TypePattern::Option(t)) => {
            deps.push(*t);
        }
        TypeKind::TypePattern(TypePattern::Result(ok, err)) => {
            deps.push(*ok);
            deps.push(*err);
        }
        TypeKind::TypePattern(TypePattern::NamedCallback(sig)) => {
            for a in &sig.arguments {
                deps.push(a.ty);
            }
            deps.push(sig.rval);
        }
        TypeKind::ReadPointer(t) | TypeKind::ReadWritePointer(t) => {
            deps.push(*t);
        }
        TypeKind::FnPointer(sig) => {
            for a in &sig.arguments {
                deps.push(a.ty);
            }
            deps.push(sig.rval);
        }
        _ => {}
    }
    deps
}
