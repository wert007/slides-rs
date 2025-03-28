use std::{ops::Index, path::PathBuf};

use binder::VariableId;
use diagnostics::{Diagnostics, Location};
use slides_rs_core::Presentation;
use string_interner::{backend::BucketBackend, symbol::SymbolUsize};

mod binder;
mod diagnostics;
mod lexer;
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileId(usize);

impl FileId {
    pub const ZERO: FileId = FileId(0);
}

struct File {
    name: PathBuf,
    content: String,
    line_breaks: Vec<usize>,
}

impl File {
    fn read(file: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&file)?;
        let line_breaks = content
            .char_indices()
            .filter(|&(_, c)| c == '\n')
            .map(|(l, _)| l)
            .collect();
        Ok(Self {
            name: file,
            content,
            line_breaks,
        })
    }

    fn content(&self) -> &str {
        &self.content
    }

    fn line_number(&self, start: usize) -> usize {
        1 + match self.line_breaks.binary_search(&start) {
            Ok(it) => it,
            Err(it) => it,
        }
    }
}

pub(crate) fn compile_project(
    file: std::path::PathBuf,
    output: std::path::PathBuf,
) -> slides_rs_core::Result<()> {
    let presentation = binder::create_presentation_from_file(file)?;
    presentation.output_to_directory(output)?;
    Ok(())
}

struct Files {
    files: Vec<File>,
}

impl Files {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    fn load_file(&mut self, path: PathBuf) -> Result<FileId, std::io::Error> {
        let index = self.files.len();
        self.files.push(File::read(path)?);
        Ok(FileId(index))
    }
}

impl Index<FileId> for Files {
    type Output = File;

    fn index(&self, index: FileId) -> &Self::Output {
        &self.files[index.0]
    }
}

impl Index<Location> for Files {
    type Output = str;

    fn index(&self, location: Location) -> &Self::Output {
        &self[location.file].content()[location.start..][..location.length]
    }
}

struct StringInterner {
    general: string_interner::StringInterner<BucketBackend<SymbolUsize>>,
    variables: string_interner::StringInterner<BucketBackend<VariableId>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            general: string_interner::StringInterner::new(),
            variables: string_interner::StringInterner::new(),
        }
    }
    pub fn resolve_variable(&self, variable_id: VariableId) -> &str {
        self.variables
            .resolve(variable_id)
            .expect("VariableId should be valid")
    }

    pub fn resolve(&self, symbol: SymbolUsize) -> &str {
        self.general
            .resolve(symbol)
            .expect("Symbol should be valid")
    }

    fn create_or_get_variable(&mut self, name: &str) -> VariableId {
        self.variables.get_or_intern(name)
    }

    fn create_or_get(&mut self, member: &str) -> SymbolUsize {
        self.general.get_or_intern(member)
    }
}

struct Context {
    presentation: Presentation,
    pub loaded_files: Files,
    diagnostics: Diagnostics,
    string_interner: StringInterner,
}

impl Context {
    fn new() -> Self {
        Self {
            presentation: Presentation::new(),
            loaded_files: Files::new(),
            diagnostics: Diagnostics::new(),
            string_interner: StringInterner::new(),
        }
    }

    fn load_file(&mut self, path: PathBuf) -> std::io::Result<FileId> {
        self.loaded_files.load_file(path)
    }
}

impl Index<FileId> for Context {
    type Output = File;

    fn index(&self, index: FileId) -> &Self::Output {
        &self.loaded_files[index]
    }
}

impl Index<Location> for Context {
    type Output = str;

    fn index(&self, location: Location) -> &Self::Output {
        &self[location.file].content()[location.start..][..location.length]
    }
}
