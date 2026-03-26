#![doc = include_str!("../README.md")]

mod codegen;
mod topo;

use interoptopus::inventory::RustInventory;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Convenience shorthand for [`Generator::new`] + [`Generator::write_file`].
pub fn generate(loader_name: &str, inv: &RustInventory, path: impl AsRef<Path>) -> Result<(), io::Error> {
    Generator::new(loader_name, inv).write_file(path)
}

/// C header code generator.
pub struct Generator<'a> {
    inv: &'a RustInventory,
    loader_name: &'a str,
    ifndef: String,
}

impl<'a> Generator<'a> {
    /// Creates a new generator.
    ///
    /// `loader_name` controls the dispatch table and loader names
    /// (e.g. `"my_lib"` produces `my_lib_api_t` and `my_lib_load`).
    #[must_use]
    pub fn new(loader_name: &'a str, inv: &'a RustInventory) -> Self {
        Self { inv, loader_name, ifndef: "interoptopus_generated".to_string() }
    }

    /// Override the `#ifndef` guard name.
    #[must_use]
    pub fn ifndef(mut self, guard: impl Into<String>) -> Self {
        self.ifndef = guard.into();
        self
    }

    /// Write the header to any [`Write`] sink.
    pub fn write_to(&self, w: &mut impl Write) -> Result<(), io::Error> {
        codegen::emit_header(w, self.inv, self.loader_name, &self.ifndef)
    }

    /// Write the header to a file, creating parent directories as needed.
    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        self.write_to(&mut file)
    }

    /// Return the header as a string.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
