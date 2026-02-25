use crate::converters::{function_name_to_c_name, to_type_specifier};
use crate::interop::ToNamingStyle;
use crate::interop::docs::write_documentation;
use crate::{DocStyle, Functions, Interop};
use interoptopus::lang::{Function, Type};
use interoptopus_backend_utils::{Error, IndentWriter, indented};

pub fn write_functions(i: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    for function in i.inventory.functions() {
        write_function(i, w, function)?;
    }

    Ok(())
}

fn write_function(i: &Interop, w: &mut IndentWriter, function: &Function) -> Result<(), Error> {
    if i.documentation == DocStyle::Inline {
        write_documentation(w, function.meta().docs())?;
    }

    match i.function_style {
        Functions::Typedefs => write_function_as_typedef_declaration(i, w, function)?,
        Functions::ForwardDeclarations => write_function_declaration(i, w, function, 999)?,
    }

    if i.documentation == DocStyle::Inline {
        w.newline()?;
    }

    Ok(())
}

pub fn write_function_declaration(i: &Interop, w: &mut IndentWriter, function: &Function, max_line: usize) -> Result<(), Error> {
    let attr = &i.function_attribute;
    let rval = to_type_specifier(i, function.signature().rval());
    let name = function_name_to_c_name(function);

    let mut params = Vec::new();

    for p in function.signature().params() {
        match p.the_type() {
            Type::Array(a) => {
                params.push(format!("{} {}[{}]", to_type_specifier(i, a.the_type()), p.name().to_naming_style(&i.function_parameter_naming), a.len(),));
            }
            _ => {
                params.push(format!("{} {}", to_type_specifier(i, p.the_type()), p.name().to_naming_style(&i.function_parameter_naming)));
            }
        }
    }

    // Test print line to see if we need to break it
    let line = format!(r"{}{} {}({});", attr, rval, name, params.join(", "));

    if line.len() <= max_line {
        indented!(w, r"{}{} {}({});", attr, rval, name, params.join(", "))?;
    } else {
        indented!(w, r"{}{} {}(", attr, rval, name)?;
        for p in params {
            indented!(w, [()], r"{}", p)?;
        }
        indented!(w, [()], r");")?;
    }

    Ok(())
}

fn function_pointer_params_string(i: &Interop, function: &Function) -> String {
    let mut params = Vec::new();
    for p in function.signature().params() {
        match p.the_type() {
            Type::Array(a) => {
                params.push(format!("{} [{}]", to_type_specifier(i, a.the_type()), a.len()));
            }
            _ => {
                params.push(to_type_specifier(i, p.the_type()).to_string());
            }
        }
    }
    params.join(", ")
}

pub fn write_dispatch_table(i: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    let loader_name = i.loader.as_deref().unwrap();
    let struct_name = format!("{loader_name}_api_t");

    indented!(w, "typedef struct {}", struct_name)?;
    indented!(w, [()], "{{")?;

    for function in i.inventory.functions() {
        let rval = to_type_specifier(i, function.signature().rval());
        let name = function_name_to_c_name(function);
        let params = function_pointer_params_string(i, function);
        indented!(w, [()()], "{} (*{})({});", rval, name, params)?;
    }

    indented!(w, [()], "}} {};", struct_name)?;

    Ok(())
}

pub fn write_loader(i: &Interop, w: &mut IndentWriter) -> Result<(), Error> {
    let loader_name = i.loader.as_deref().unwrap();
    let struct_name = format!("{loader_name}_api_t");
    let fn_name = format!("{loader_name}_load");

    // Windows loader
    indented!(w, "#if defined(_WIN32)")?;
    indented!(w, "#include <windows.h>")?;
    indented!(w, "static int {}(const char* path, {}* api)", fn_name, struct_name)?;
    indented!(w, [()], "{{")?;
    indented!(w, [()()], "HMODULE lib = LoadLibraryA(path);")?;
    indented!(w, [()()], "if (!lib) return -1;")?;

    for function in i.inventory.functions() {
        let rval = to_type_specifier(i, function.signature().rval());
        let name = function_name_to_c_name(function);
        let params = function_pointer_params_string(i, function);
        indented!(w, [()()], "api->{} = ({} (*)({}))(void*)GetProcAddress(lib, \"{}\");", name, rval, params, name)?;
        indented!(w, [()()], "if (!api->{}) return -1;", name)?;
    }

    indented!(w, [()()], "return 0;")?;
    indented!(w, [()], "}}")?;

    // POSIX loader
    indented!(w, "#else")?;
    indented!(w, "#include <dlfcn.h>")?;
    indented!(w, "#include <string.h>")?;
    indented!(w, "static int {}(const char* path, {}* api)", fn_name, struct_name)?;
    indented!(w, [()], "{{")?;
    indented!(w, [()()], "void* lib = dlopen(path, RTLD_NOW);")?;
    indented!(w, [()()], "if (!lib) return -1;")?;
    indented!(w, [()()], "void* sym;")?;

    for function in i.inventory.functions() {
        let name = function_name_to_c_name(function);
        indented!(w, [()()], "sym = dlsym(lib, \"{}\");", name)?;
        indented!(w, [()()], "if (!sym) return -1;")?;
        indented!(w, [()()], "memcpy(&api->{}, &sym, sizeof(sym));", name)?;
    }

    indented!(w, [()()], "return 0;")?;
    indented!(w, [()], "}}")?;
    indented!(w, "#endif")?;

    Ok(())
}

fn write_function_as_typedef_declaration(i: &Interop, w: &mut IndentWriter, function: &Function) -> Result<(), Error> {
    let rval = to_type_specifier(i, function.signature().rval());
    let name = function_name_to_c_name(function);

    let mut params = Vec::new();

    for p in function.signature().params() {
        match p.the_type() {
            Type::Array(a) => {
                params.push(format!("{} [{}]", to_type_specifier(i, a.the_type()), a.len(),));
            }
            _ => {
                params.push(to_type_specifier(i, p.the_type()).to_string());
            }
        }
    }
    indented!(w, r"typedef {} (*{})({});", rval, name, params.join(", "))?;

    Ok(())
}
