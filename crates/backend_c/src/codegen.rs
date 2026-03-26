//! Core code generation routines for emitting C header constructs.

use crate::topo;
use interoptopus::inventory::{RustInventory, TypeId};
use interoptopus::lang::function::{Function, Signature};
use interoptopus::lang::types::{Enum, Primitive, Struct, Type, TypeKind, TypePattern, VariantKind};
use std::io::{self, Write};

/// Emit a complete C header into `w`.
pub fn emit_header(w: &mut impl Write, inv: &RustInventory, loader_name: &str, ifndef: &str) -> Result<(), io::Error> {
    let cx = Cx { inv };
    let functions = sorted_functions(inv);

    writeln!(w, "#ifndef {ifndef}")?;
    writeln!(w, "#define {ifndef}")?;
    writeln!(w)?;
    writeln!(w, "#ifdef __cplusplus")?;
    writeln!(w, "extern \"C\" {{")?;
    writeln!(w, "#endif")?;
    writeln!(w)?;
    writeln!(w, "#include <stdint.h>")?;
    writeln!(w, "#include <stdbool.h>")?;
    writeln!(w)?;

    for tid in &topo::sort_types(inv) {
        cx.emit_type(w, &inv.types[tid])?;
    }
    writeln!(w)?;

    for f in &functions {
        cx.emit_function_decl(w, f)?;
    }
    writeln!(w)?;

    emit_dispatch_table(w, &cx, loader_name, &functions)?;
    writeln!(w)?;
    emit_dynamic_loader(w, &cx, loader_name, &functions)?;
    emit_static_loader(w, loader_name, &functions)?;

    writeln!(w, "#ifdef __cplusplus")?;
    writeln!(w, "}}")?;
    writeln!(w, "#endif")?;
    writeln!(w)?;
    writeln!(w, "#endif /* {ifndef} */")?;
    Ok(())
}

/// Turn a Rust type name (e.g. `Option<Vec2>`) into a valid C identifier (`OPTIONVEC2`).
fn c_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn primitive_name(p: Primitive) -> &'static str {
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

struct Cx<'a> {
    inv: &'a RustInventory,
}

impl Cx<'_> {
    fn resolve(&self, tid: &TypeId) -> &Type {
        &self.inv.types[tid]
    }

    fn type_name(&self, tid: &TypeId) -> String {
        let ty = self.resolve(tid);
        match &ty.kind {
            TypeKind::Primitive(p) => primitive_name(*p).to_string(),
            TypeKind::ReadPointer(inner) => format!("const {}*", self.type_name(inner)),
            TypeKind::ReadWritePointer(inner) => format!("{}*", self.type_name(inner)),
            TypeKind::TypePattern(TypePattern::CChar) => "char".to_string(),
            TypeKind::TypePattern(TypePattern::Bool) => "bool".to_string(),
            TypeKind::TypePattern(TypePattern::CVoid) => "void".to_string(),
            _ => c_name(&ty.name),
        }
    }

    /// Returns the C type specifier, mapping `Primitive::Void` to `"void"`.
    fn type_spec(&self, tid: &TypeId) -> String {
        let ty = self.resolve(tid);
        if matches!(ty.kind, TypeKind::Primitive(Primitive::Void)) {
            "void".to_string()
        } else {
            self.type_name(tid)
        }
    }

    fn param_types(&self, sig: &Signature) -> String {
        if sig.arguments.is_empty() {
            return "void".to_string();
        }
        sig.arguments.iter().map(|a| self.type_spec(&a.ty)).collect::<Vec<_>>().join(", ")
    }

    fn param_list(&self, sig: &Signature) -> String {
        if sig.arguments.is_empty() {
            return "void".to_string();
        }
        sig.arguments
            .iter()
            .map(|a| format!("{} {}", self.type_spec(&a.ty), a.name.to_uppercase()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn emit_type(&self, w: &mut impl Write, ty: &Type) -> Result<(), io::Error> {
        let name = c_name(&ty.name);
        match &ty.kind {
            TypeKind::Primitive(_)
            | TypeKind::ReadPointer(_)
            | TypeKind::ReadWritePointer(_)
            | TypeKind::TypePattern(TypePattern::Bool | TypePattern::CChar | TypePattern::CVoid) => {}

            TypeKind::Struct(s) => emit_struct(w, self, &name, s)?,
            TypeKind::Enum(e) => emit_enum(w, self, &name, e)?,
            TypeKind::TypePattern(TypePattern::NamedCallback(sig)) => emit_callback(w, self, &name, sig)?,

            TypeKind::TypePattern(TypePattern::Slice(inner) | TypePattern::SliceMut(inner)) => {
                let is_mut = matches!(ty.kind, TypeKind::TypePattern(TypePattern::SliceMut(_)));
                let const_q = if is_mut { "" } else { "const " };
                let inner = self.type_spec(inner);
                writeln!(w, "typedef struct {name}\n{{\n    {const_q}{inner}* data;\n    uint64_t len;\n}} {name};\n")?;
            }

            TypeKind::TypePattern(TypePattern::Vec(inner)) => {
                let inner = self.type_spec(inner);
                writeln!(w, "typedef struct {name}\n{{\n    {inner}* ptr;\n    uint64_t len;\n    uint64_t capacity;\n}} {name};\n")?;
            }

            TypeKind::TypePattern(TypePattern::Utf8String) => {
                writeln!(w, "typedef struct {name}\n{{\n    uint8_t* ptr;\n    uint64_t len;\n    uint64_t capacity;\n}} {name};\n")?;
            }

            TypeKind::TypePattern(TypePattern::Option(inner)) => emit_option(w, self, &name, inner)?,
            TypeKind::TypePattern(TypePattern::Result(ok, err)) => emit_result(w, self, &name, ok, err)?,

            TypeKind::FnPointer(sig) => {
                let rval = self.type_spec(&sig.rval);
                let params = self.param_types(sig);
                writeln!(w, "typedef {rval} (*{name})({params});\n")?;
            }

            TypeKind::Opaque | TypeKind::Service => {
                writeln!(w, "typedef struct {name} {name};\n")?;
            }

            _ => {}
        }
        Ok(())
    }

    fn emit_function_decl(&self, w: &mut impl Write, f: &Function) -> Result<(), io::Error> {
        writeln!(w, "{} {}({});", self.type_spec(&f.signature.rval), f.name, self.param_list(&f.signature))
    }
}

fn emit_struct(w: &mut impl Write, cx: &Cx<'_>, name: &str, s: &Struct) -> Result<(), io::Error> {
    writeln!(w, "typedef struct {name}")?;
    writeln!(w, "{{")?;
    for f in &s.fields {
        if let TypeKind::Array(arr) = &cx.resolve(&f.ty).kind {
            writeln!(w, "    {} {}[{}];", cx.type_spec(&arr.ty), f.name, arr.len)?;
        } else {
            writeln!(w, "    {} {};", cx.type_spec(&f.ty), f.name)?;
        }
    }
    writeln!(w, "}} {name};\n")
}

fn emit_enum(w: &mut impl Write, cx: &Cx<'_>, name: &str, e: &Enum) -> Result<(), io::Error> {
    let has_data = e.variants.iter().any(|v| matches!(v.kind, VariantKind::Tuple(_)));

    if has_data {
        let tag = format!("{name}_TAG");
        writeln!(w, "typedef enum {tag}")?;
        writeln!(w, "{{")?;
        for (i, v) in e.variants.iter().enumerate() {
            writeln!(w, "    {}_{} = {i},", name, v.name.to_uppercase())?;
        }
        writeln!(w, "}} {tag};\n")?;

        writeln!(w, "typedef struct {name}")?;
        writeln!(w, "{{")?;
        writeln!(w, "    {tag} tag;")?;
        writeln!(w, "    union")?;
        writeln!(w, "    {{")?;
        for v in &e.variants {
            if let VariantKind::Tuple(tid) = &v.kind {
                writeln!(w, "        {} {};", cx.type_spec(tid), v.name.to_lowercase())?;
            }
        }
        writeln!(w, "    }};")?;
    } else {
        writeln!(w, "typedef enum {name}")?;
        writeln!(w, "{{")?;
        for (i, v) in e.variants.iter().enumerate() {
            writeln!(w, "    {}_{} = {i},", name, v.name.to_uppercase())?;
        }
    }
    writeln!(w, "}} {name};\n")
}

fn emit_option(w: &mut impl Write, cx: &Cx<'_>, name: &str, inner: &TypeId) -> Result<(), io::Error> {
    let tag = format!("{name}_TAG");
    let inner = cx.type_spec(inner);
    writeln!(w, "typedef enum {tag}\n{{\n    {name}_SOME = 0,\n    {name}_NONE = 1,\n}} {tag};\n")?;
    writeln!(w, "typedef struct {name}\n{{\n    {tag} tag;\n    union\n    {{\n        {inner} some;\n    }};\n}} {name};\n")
}

fn emit_result(w: &mut impl Write, cx: &Cx<'_>, name: &str, ok: &TypeId, err: &TypeId) -> Result<(), io::Error> {
    let tag = format!("{name}_TAG");
    let ok = cx.type_spec(ok);
    let err = cx.type_spec(err);
    writeln!(w, "typedef enum {tag}\n{{\n    {name}_OK = 0,\n    {name}_ERR = 1,\n}} {tag};\n")?;
    writeln!(w, "typedef struct {name}\n{{\n    {tag} tag;\n    union\n    {{\n        {ok} ok;\n        {err} err;\n    }};\n}} {name};\n")
}

fn emit_callback(w: &mut impl Write, cx: &Cx<'_>, name: &str, sig: &Signature) -> Result<(), io::Error> {
    let rval = cx.type_spec(&sig.rval);
    let mut params: Vec<String> = sig.arguments.iter().map(|a| cx.type_spec(&a.ty)).collect();
    params.push("const void*".to_string());
    let fn_typedef = format!("{name}_fn");
    writeln!(w, "typedef {rval} (*{fn_typedef})({});\n", params.join(", "))?;
    writeln!(w, "typedef struct {name}")?;
    writeln!(w, "{{")?;
    writeln!(w, "    {fn_typedef} callback;")?;
    writeln!(w, "    const void* data;")?;
    writeln!(w, "    void (*destructor)(const void*);")?;
    writeln!(w, "}} {name};\n")
}

fn emit_dispatch_table(w: &mut impl Write, cx: &Cx<'_>, loader_name: &str, functions: &[&Function]) -> Result<(), io::Error> {
    let name = format!("{loader_name}_api_t");
    writeln!(w, "typedef struct {name}")?;
    writeln!(w, "{{")?;
    for f in functions {
        writeln!(w, "    {} (*{})({});", cx.type_spec(&f.signature.rval), f.name, cx.param_types(&f.signature))?;
    }
    writeln!(w, "}} {name};")
}

fn emit_dynamic_loader(w: &mut impl Write, cx: &Cx<'_>, loader_name: &str, functions: &[&Function]) -> Result<(), io::Error> {
    let api_t = format!("{loader_name}_api_t");
    let load = format!("{loader_name}_load");

    writeln!(w, "#if defined(_WIN32)")?;
    writeln!(w, "#include <windows.h>")?;
    writeln!(w, "static int {load}(const char* path, {api_t}* api)")?;
    writeln!(w, "{{")?;
    writeln!(w, "    HMODULE lib = LoadLibraryA(path);")?;
    writeln!(w, "    if (!lib) return -1;")?;
    for f in functions {
        let n = &f.name;
        let cast = format!("{} (*)({}))", cx.type_spec(&f.signature.rval), cx.param_types(&f.signature));
        writeln!(w, "    api->{n} = ({cast}(void*)GetProcAddress(lib, \"{n}\");")?;
        writeln!(w, "    if (!api->{n}) return -1;")?;
    }
    writeln!(w, "    return 0;")?;
    writeln!(w, "}}")?;

    writeln!(w, "#else")?;
    writeln!(w, "#include <dlfcn.h>")?;
    writeln!(w, "#include <string.h>")?;
    writeln!(w, "static int {load}(const char* path, {api_t}* api)")?;
    writeln!(w, "{{")?;
    writeln!(w, "    void* lib = dlopen(path, RTLD_NOW);")?;
    writeln!(w, "    if (!lib) return -1;")?;
    writeln!(w, "    void* sym;")?;
    for f in functions {
        let n = &f.name;
        writeln!(w, "    sym = dlsym(lib, \"{n}\");")?;
        writeln!(w, "    if (!sym) return -1;")?;
        writeln!(w, "    memcpy(&api->{n}, &sym, sizeof(sym));")?;
    }
    writeln!(w, "    return 0;")?;
    writeln!(w, "}}")?;
    writeln!(w, "#endif")?;
    writeln!(w)
}

fn emit_static_loader(w: &mut impl Write, loader_name: &str, functions: &[&Function]) -> Result<(), io::Error> {
    let api_t = format!("{loader_name}_api_t");
    let load = format!("{loader_name}_load_static");
    let guard = format!("{}_STATIC", loader_name.to_uppercase());

    writeln!(w, "#ifdef {guard}")?;
    writeln!(w, "static int {load}({api_t}* api)")?;
    writeln!(w, "{{")?;
    for f in functions {
        writeln!(w, "    api->{n} = {n};", n = f.name)?;
    }
    writeln!(w, "    return 0;")?;
    writeln!(w, "}}")?;
    writeln!(w, "#endif /* {guard} */")?;
    writeln!(w)
}
