use std::{path::PathBuf, str::FromStr};

use diagnostics::Diagnostics;
use slides_rs_core::{Presentation, PresentationEmitter};

pub mod binder;
pub mod diagnostics;
pub mod evaluator;
pub mod lexer;
pub mod module;
pub mod parser;

#[derive(Debug, Clone, Copy, Default)]
pub struct DebugLang {
    pub tokens: bool,
    pub parser: bool,
    pub binder: bool,
    pub scopes: bool,
    pub presentation: bool,
}

impl FromStr for DebugLang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self::default();
        for part in s.split(',') {
            let part = part.trim().to_lowercase();
            if part.is_empty() {
                continue;
            }
            match part.as_str() {
                "t" | "tok" | "token" | "tokens" => result.tokens = true,
                "p" | "par" | "parse" | "parser" => result.parser = true,
                "b" | "bin" | "bind" | "binder" => result.binder = true,
                "s" | "scop" | "scope" => result.scopes = true,
                "pres" | "presentation" => result.presentation = true,
                _unknown_field => {
                    return Err(format!(
                        "Following parts can be debugged [tokens], [parser], [binder], [scope] or [presentation] you can use shortened names as well and combine multiple by comma"
                    ));
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct CompilationResult {
    pub diagnostics: Diagnostics,
    pub used_files: Vec<PathBuf>,
}

pub fn compile_project(
    file: impl Into<std::path::PathBuf>,
    output: impl Into<std::path::PathBuf>,
    debug: DebugLang,
) -> slides_rs_core::Result<CompilationResult> {
    let file = file.into();
    let output = output.into();
    let mut result = CompilationResult {
        diagnostics: Diagnostics::new(),
        used_files: vec![file.clone()],
    };
    let presentation = match binder::create_presentation_from_file(file, debug) {
        Ok(it) => it,
        Err(binder::Error::LanguageErrors(diagnostics)) => {
            result.diagnostics = diagnostics;
            Presentation::new()
        }
        Err(binder::Error::IoError(err)) => return Err(err.into()),
        Err(binder::Error::SlideError(err)) => return Err(err.into()),
    };
    result
        .used_files
        .extend_from_slice(presentation.used_files());
    if debug.presentation {
        dbg!(&presentation);
    }
    let mut emitter = PresentationEmitter::new(output)?;
    presentation.output_to_directory(&mut emitter)?;
    result
        .used_files
        .extend_from_slice(emitter.referenced_files());
    Ok(result)
}
