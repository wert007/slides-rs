use std::path::PathBuf;

use slides_rs_core::Presentation;

use super::{
    Context, File,
    parser::{self, debug_ast},
};

pub(crate) fn create_presentation_from_file(file: PathBuf) -> slides_rs_core::Result<Presentation> {
    let mut context = Context::new();
    let file = context.load_file(file)?;
    let ast = parser::parse_file(file, &mut context);
    debug_ast(&ast, &context);
    bind_ast(ast, &mut context);
    let Context {
        presentation,
        diagnostics,
        loaded_files,
    } = context;
    if !diagnostics.is_empty() {
        diagnostics.write(&mut std::io::stdout(), &loaded_files)?;
    }
    Ok(presentation)
}

fn bind_ast(ast: parser::Ast, context: &mut Context) {}
