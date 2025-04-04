use std::{fs::File, io::Write, path::PathBuf};

use crate::{BASE_STYLE, NAVIGATION_JS};

pub struct PresentationEmitter<W: Write> {
    directory: PathBuf,
    html: W,
    css: W,
    js: W,
    referenced_files: Vec<PathBuf>,
}

impl PresentationEmitter<File> {
    pub fn new(directory: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&directory)?;
        let html = File::create(directory.join("index.html"))?;
        let mut css = File::create(directory.join("style.css"))?;
        writeln!(css, "{BASE_STYLE}")?;
        let mut js = File::create(directory.join("navigation.js"))?;
        writeln!(js, "{NAVIGATION_JS}")?;

        Ok(Self {
            directory,
            html,
            css,
            js,
            referenced_files: Vec::new(),
        })
    }

    pub fn copy_referenced_files(&self) -> std::io::Result<()> {
        for file in &self.referenced_files {
            let to = self.directory.join(file);
            if to.exists() {
                // Make file checks later!
                continue;
            }
            _ = to.parent().map(|p| std::fs::create_dir(p)).transpose();
            std::fs::copy(file, to)?;
        }
        Ok(())
    }
}

impl<W: Write> PresentationEmitter<W> {
    pub fn raw_html(&mut self) -> &mut W {
        &mut self.html
    }

    pub fn raw_css(&mut self) -> &mut W {
        &mut self.css
    }

    pub fn raw_js(&mut self) -> &mut W {
        &mut self.js
    }

    pub fn add_file(&mut self, path: impl Into<PathBuf>) -> std::io::Result<()> {
        let path = path.into();
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ));
        }
        self.referenced_files.push(path);
        Ok(())
    }
}
