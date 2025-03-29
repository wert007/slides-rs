use crate::{Context, compiler};

pub fn format_file(file: std::path::PathBuf) -> std::io::Result<()> {
    let mut context = Context::new();
    let file = context.load_file(file)?;
    let context = compiler::parser::parse_file(file, &mut context);
    Ok(())
}
