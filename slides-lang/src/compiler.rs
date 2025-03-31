pub mod binder;
pub mod diagnostics;
mod evaluator;
pub mod lexer;
pub mod parser;

pub fn compile_project(
    file: std::path::PathBuf,
    output: std::path::PathBuf,
) -> slides_rs_core::Result<()> {
    let presentation = binder::create_presentation_from_file(file)?;
    presentation.output_to_directory(output)?;
    Ok(())
}
