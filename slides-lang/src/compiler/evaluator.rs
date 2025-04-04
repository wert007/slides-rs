use std::collections::BTreeMap;

use slides_rs_core::{
    DynamicElementStyling, FilePlacement, ImageStyling, LabelStyling, Slide, SlideStyling,
};

use super::binder::{BoundAst, BoundNode, BoundNodeKind};
use crate::{Context, VariableId};

pub mod functions;
mod slide;
mod style;
mod value;
pub use value::*;

struct Scope {
    variables: BTreeMap<VariableId, Value>,
}

impl Scope {
    pub fn global() -> Self {
        Self {
            variables: BTreeMap::new(),
        }
    }
    pub fn new() -> Self {
        Self {
            variables: BTreeMap::new(),
        }
    }

    fn set_variable(&mut self, name: VariableId, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_variable(&self, name: VariableId) -> Option<&Value> {
        self.variables.get(&name)
    }

    fn get_variable_mut(&mut self, name: VariableId) -> Option<&mut Value> {
        self.variables.get_mut(&name)
    }
}

struct Evaluator {
    scopes: Vec<Scope>,
    slide: Option<Slide>,
    styling: Option<DynamicElementStyling>,
}
impl Evaluator {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::global()],
            slide: None,
            styling: None,
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

    fn set_variable(&mut self, name: VariableId, value: Value) {
        self.current_scope().set_variable(name, value);
    }

    fn get_variable_mut(&mut self, name: VariableId) -> &mut Value {
        self.scopes
            .iter_mut()
            .rev()
            .filter_map(|s| s.get_variable_mut(name))
            .next()
            .expect("Variable exists")
    }

    fn get_variable(&self, name: VariableId) -> &Value {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get_variable(name))
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
    // dbg!(&context.presentation);
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
        BoundNodeKind::ElementStatement(element_statement) => {
            evaluate_element_statement(element_statement, evaluator, context)
        }
        BoundNodeKind::ImportStatement(import_statement) => {
            evaluate_import_statement(import_statement, evaluator, context)
        }
        err => unreachable!("No Top Level Statement: {err:?}"),
    }
}

fn evaluate_import_statement(
    import_statement: std::path::PathBuf,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let path_extensions = import_statement.to_str().unwrap().split('.').rev();
    enum State {
        Unknown,
        HtmlUnknown,
        HtmlHead,
    }

    impl State {
        pub fn is_finished(&self) -> bool {
            matches!(self, Self::HtmlHead)
        }
    }
    let mut state = State::Unknown;
    for extension in path_extensions {
        match extension {
            "html" => state = State::HtmlUnknown,
            "head" => state = State::HtmlHead,
            missing => unreachable!("Missing {missing}"),
        }
        if state.is_finished() {
            break;
        }
    }
    match state {
        State::HtmlHead => {
            context
                .presentation
                .add_extern_file(FilePlacement::HtmlHead, import_statement)?;
        }
        State::Unknown | State::HtmlUnknown => unreachable!(),
    }
    Ok(())
}

fn evaluate_element_statement(
    element_statement: super::binder::ElementStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let parameters = element_statement.parameters;
    evaluator.set_variable(
        element_statement.name,
        Value::UserFunction(UserFunctionValue {
            parameters,
            body: element_statement.body,
            return_type: element_statement.type_,
        }),
    );
    Ok(())
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
    evaluator.slide = Some(slide);
    slide::evaluate_to_slide(slide_statement.body, evaluator, context)?;
    context
        .presentation
        .add_slide(evaluator.slide.take().expect("there to be slide"));
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
    let styling = match styling_statement.type_ {
        super::binder::StylingType::Label => LabelStyling::new().to_dynamic(name.clone()),
        super::binder::StylingType::Image => ImageStyling::new().to_dynamic(name.clone()),
        super::binder::StylingType::Slide => SlideStyling::new().to_dynamic(name.clone()),
    };
    evaluator.styling = Some(styling);
    style::evaluate_to_styling(styling_statement.body, evaluator, context);
    let styling = evaluator.styling.take().expect("styilng");
    let reference = context.presentation.add_dynamic_styling(styling);
    evaluator.set_variable(styling_statement.name, Value::StyleReference(reference));
    Ok(())
}
