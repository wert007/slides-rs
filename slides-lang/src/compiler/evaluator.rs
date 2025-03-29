use std::collections::HashMap;

use slides_rs_core::{ElementStyling, ImageStyling, LabelStyling, Slide, SlideStyling, ToCss};

use super::{
    Context,
    binder::{BoundAst, BoundNode, BoundNodeKind, Value},
};

pub mod functions;
mod slide;
mod style;

struct Scope {
    variables: HashMap<String, Value>,
}

impl Scope {
    pub fn global() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_variable_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.variables.get_mut(name)
    }
}

struct Evaluator {
    scopes: Vec<Scope>,
}
impl Evaluator {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::global()],
        }
    }

    fn push_scope(&mut self) -> &mut Scope {
        self.scopes.push(Scope::new());
        self.current_scope()
    }

    fn drop_scope(&mut self) -> Scope {
        self.scopes.pop().expect("global scope missing")
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("global scope missing")
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.current_scope().set_variable(name, value);
    }

    fn get_variable_mut(&mut self, name: &str) -> &mut Value {
        self.scopes
            .iter_mut()
            .rev()
            .filter_map(|s| s.get_variable_mut(name))
            .next()
            .expect("Variable exists")
    }
}

pub fn create_presentation_from_ast(
    ast: BoundAst,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let mut evaluator = Evaluator::new();

    for statement in ast.statements {
        evaluate_statement(statement, &mut evaluator, context)?;
    }
    Ok(())
}

fn evaluate_statement(
    statement: BoundNode,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    match statement.kind {
        BoundNodeKind::Error(()) => unreachable!("Errors should create errors!"),
        BoundNodeKind::StylingStatement(styling_statement) => {
            evaluate_styling_statement(styling_statement, evaluator, context)
        }
        BoundNodeKind::SlideStatement(slide_statement) => {
            evaluate_slide_statement(slide_statement, evaluator, context)
        }
        err => unreachable!("No Top Level Statement: {err:?}"),
    }
}

fn evaluate_slide_statement(
    slide_statement: super::binder::SlideStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let slide = Slide::new().with_name(
        context
            .string_interner
            .resolve_variable(slide_statement.name),
    );
    let slide = slide::evaluate_to_slide(slide, slide_statement.body, evaluator, context)?;
    context.presentation.add_slide(slide);
    Ok(())
}

fn evaluate_styling_statement(
    styling_statement: super::binder::StylingStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let name = context
        .string_interner
        .resolve_variable(styling_statement.name)
        .to_owned();
    let reference = match styling_statement.type_ {
        super::binder::StylingType::Label => {
            let styling = style::evaluate_to_styling(
                LabelStyling::new(),
                styling_statement.body,
                evaluator,
                context,
            );
            context.presentation.add_styling(styling, &name)
        }
        super::binder::StylingType::Image => {
            let styling = style::evaluate_to_styling(
                ImageStyling::new(),
                styling_statement.body,
                evaluator,
                context,
            );
            context.presentation.add_styling(styling, &name)
        }
        super::binder::StylingType::Slide => {
            let styling = style::evaluate_to_styling(
                SlideStyling::new(),
                styling_statement.body,
                evaluator,
                context,
            );
            context.presentation.add_styling(styling, &name)
        }
    };
    evaluator.set_variable(name, Value::StyleReference(reference));
    Ok(())
}
