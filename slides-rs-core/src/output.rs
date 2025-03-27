use std::{fs::File, io::Write, path::PathBuf};

use crate::BASE_STYLE;

pub struct PresentationEmitter<W: Write> {
    html: W,
    css: W,
}

impl PresentationEmitter<File> {
    pub fn new(directory: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&directory)?;
        let html = File::create(directory.join("index.html"))?;
        let mut css = File::create(directory.join("style.css"))?;
        writeln!(css, "{BASE_STYLE}")?;

        Ok(Self { html, css })
    }
}

impl<W: Write> PresentationEmitter<W> {
    pub fn raw_html(&mut self) -> &mut W {
        &mut self.html
    }

    pub fn raw_css(&mut self) -> &mut W {
        &mut self.css
    }
}
