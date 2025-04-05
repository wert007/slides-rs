use std::str::FromStr;

pub mod binder;
pub mod diagnostics;
pub mod evaluator;
pub mod lexer;
pub mod parser;

#[derive(Debug, Clone, Copy, Default)]
pub struct DebugLang {
    pub tokens: bool,
    pub parser: bool,
    pub binder: bool,
    pub presentation: bool,
}

impl FromStr for DebugLang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self::default();
        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            match part {
                "t" | "tok" | "token" | "tokens" => result.tokens = true,
                "p" | "par" | "parse" | "parser" => result.parser = true,
                "b" | "bin" | "bind" | "binder" => result.binder = true,
                "pres" | "presentation" => result.presentation = true,
                unknown_field => return Err(unknown_field.into()),
            }
        }
        Ok(result)
    }
}

pub fn compile_project(
    file: std::path::PathBuf,
    output: std::path::PathBuf,
    debug: DebugLang,
) -> slides_rs_core::Result<()> {
    let presentation = binder::create_presentation_from_file(file, debug)?;
    if debug.presentation {
        dbg!(&presentation);
    }
    presentation.output_to_directory(output)?;
    Ok(())
}
