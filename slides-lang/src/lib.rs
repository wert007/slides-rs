#![feature(lock_value_accessors)]
use std::{
    cell::RefCell,
    ops::Index,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use compiler::{DebugLang, binder::typing::TypeInterner, diagnostics::Diagnostics, module::Module};
use slides_rs_core::Presentation;
use string_interner::{Symbol, backend::BucketBackend, symbol::SymbolUsize};

pub mod compiler;
pub mod formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub start: usize,
    pub length: usize,
}
impl Location {
    pub fn set_end(&mut self, end: usize) {
        self.length = end - self.start;
    }

    pub(crate) fn combine(start: Location, end: Location) -> Self {
        Self {
            file: start.file,
            start: start.start,
            length: end.end() - start.start,
        }
    }

    fn end(&self) -> usize {
        self.start + self.length
    }

    pub const fn zero() -> Location {
        Self {
            file: FileId::ZERO,
            start: 0,
            length: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileId(usize);

impl FileId {
    pub const ZERO: FileId = FileId(0);
}

pub struct File {
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

pub struct Files {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct VariableId(usize);

impl Symbol for VariableId {
    fn try_from_usize(index: usize) -> Option<Self> {
        Some(Self(index))
    }

    fn to_usize(self) -> usize {
        self.0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModuleIndex(usize);

impl ModuleIndex {
    pub const ANY: Self = ModuleIndex(0);
}

pub struct Modules {
    directory: PathBuf,
    modules: Vec<Arc<RwLock<Module>>>,
}

impl Modules {
    pub fn new() -> Self {
        Self {
            directory: "./slides-modules/".into(),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) -> ModuleIndex {
        self.modules.push(Arc::new(RwLock::new(module)));
        ModuleIndex(self.modules.len())
    }
}

impl Index<ModuleIndex> for Modules {
    type Output = Arc<RwLock<Module>>;

    fn index(&self, index: ModuleIndex) -> &Self::Output {
        if index.0 == 0 {
            panic!("ModuleIndex::ANY is not a valid index");
        }
        &self.modules[index.0 - 1]
    }
}

pub struct Context {
    presentation: Arc<RwLock<Presentation>>,
    pub loaded_files: Files,
    diagnostics: Diagnostics,
    string_interner: StringInterner,
    type_interner: TypeInterner,
    debug: DebugLang,
    modules: Modules,
}

impl Context {
    fn new() -> Self {
        Self {
            presentation: Arc::new(RwLock::new(Presentation::new())),
            loaded_files: Files::new(),
            diagnostics: Diagnostics::new(),
            string_interner: StringInterner::new(),
            type_interner: TypeInterner::new(),
            debug: DebugLang::default(),
            modules: Modules::new(),
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
