use std::{ops::Index, path::PathBuf};

use lexer::Location;
use slides_rs_core::Presentation;

mod binder;
mod lexer;
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileId(usize);

struct File {
    name: PathBuf,
    content: String,
}

impl File {
    fn read(file: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&file)?;
        Ok(Self {
            name: file,
            content,
        })
    }

    fn content(&self) -> &str {
        &self.content
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

struct Context {
    presentation: Presentation,
    loaded_files: Vec<File>,
}

impl Context {
    fn new() -> Self {
        Self {
            presentation: Presentation::new(),
            loaded_files: Vec::new(),
        }
    }

    fn load_file(&mut self, path: PathBuf) -> std::io::Result<FileId> {
        let index = self.loaded_files.len();
        self.loaded_files.push(File::read(path)?);
        Ok(FileId(index))
    }
}

impl Index<FileId> for Context {
    type Output = File;

    fn index(&self, index: FileId) -> &Self::Output {
        &self.loaded_files[index.0]
    }
}

impl Index<Location> for Context {
    type Output = str;

    fn index(&self, location: Location) -> &Self::Output {
        &self[location.file].content()[location.start..][..location.length]
    }
}
