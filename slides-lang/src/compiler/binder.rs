use std::path::PathBuf;

use slides_rs_core::Presentation;

use super::{
    Context, File,
    diagnostics::Location,
    parser::{self, SyntaxNode, debug_ast},
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

struct Binder {}

impl Binder {
    pub fn new() -> Self {
        Self {}
    }
}

enum BoundNodeKind {}

struct BoundNode {
    base: Option<SyntaxNode>,
    location: Location,
    kind: BoundNodeKind,
}

struct BoundAst {
    statements: Vec<BoundNode>,
}

fn bind_ast(ast: parser::Ast, context: &mut Context) -> BoundAst {
    let mut binder = Binder::new();
    let mut statements = Vec::with_capacity(ast.statements.len());
    for statement in ast.statements {
        statements.push(bind_statement(statement, &mut binder, context));
    }
    BoundAst { statements }
}

fn bind_statement(statement: SyntaxNode, binder: &mut Binder, context: &mut Context) -> BoundNode {
    todo!()
}
