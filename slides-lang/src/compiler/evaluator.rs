use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use index_map::IndexMap;
use slides_rs_core::{
    DynamicElementStyling, FilePlacement, ImageStyling, LabelStyling, Slide, SlideStyling,
    TextStyling,
};

use super::binder::{BoundAst, BoundError, BoundNode, BoundNodeKind, StylingType, typing::TypeId};
use crate::{Context, Location, VariableId, compiler::evaluator::value::UserFunctionValue};

pub mod functions;
mod slide;
mod style;
pub mod value;
// pub use value::*;

#[derive(Debug, Clone)]
pub struct Value {
    pub value: value::Value,
    pub location: Location,
}

struct Scope {
    values: IndexMap<Value>,
    variables: HashMap<VariableId, usize>,
}

impl Scope {
    pub fn global() -> Self {
        Self {
            values: IndexMap::new(),
            variables: HashMap::new(),
        }
    }
    pub fn new() -> Self {
        Self {
            values: IndexMap::new(),
            variables: HashMap::new(),
        }
    }

    fn set_variable(&mut self, name: VariableId, value: Value) {
        let index = self.values.insert(value);
        self.variables.insert(name, index);
    }

    fn get_variable(&self, name: VariableId) -> Option<&Value> {
        let index = self.variables.get(&name)?;
        self.values.get(*index)
    }

    fn get_variable_mut(&mut self, name: VariableId) -> Option<&mut Value> {
        let index = self.variables.get(&name)?;
        self.values.get_mut(*index)
    }

    fn variables(self) -> impl Iterator<Item = (VariableId, Value)> {
        self.values.into_iter().filter_map(move |(vindex, v)| {
            Some((
                *self
                    .variables
                    .iter()
                    .find(|(_, index)| vindex == **index)
                    .map(|(v, _)| v)?,
                v,
            ))
        })
    }
}

pub struct Exception {
    location: Location,
    message: String,
}
impl Exception {
    fn print(&self, loaded_files: &crate::Files) {
        eprintln!(
            "Exception in [{}:{}]: \"{}\"",
            loaded_files[self.location.file].name.display(),
            loaded_files[self.location.file].line_number(self.location.start),
            self.message
        );
        eprintln!("Stopping execution.");
    }
}

pub struct Evaluator {
    scopes: Vec<Scope>,
    slide: Option<Slide>,
    styling: Option<DynamicElementStyling>,
    exception: Option<Exception>,
    default_template: Option<value::UserFunctionValue>,
    return_value: Option<Value>,
}
impl Evaluator {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::global()],
            slide: None,
            styling: None,
            exception: None,
            default_template: None,
            return_value: None,
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

    #[track_caller]
    fn get_variable(&self, name: VariableId) -> &Value {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get_variable(name))
            .next()
            .expect("Variable exists")
    }

    fn try_get_variable(&self, name: VariableId) -> Option<&Value> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.get_variable(name))
            .next()
    }

    fn ensure_unsigned(&mut self, value: Value) -> usize {
        let number = value.value.into_integer();
        if number < 0 {
            self.exception = Some(Exception {
                location: value.location,
                message: "Invalid negative value".into(),
            });
            0
        } else {
            number as usize
        }
    }

    fn ensure_unsigned_float(&mut self, value: Value) -> f64 {
        let number = value.value.into_float();
        if number < 0.0 {
            self.exception = Some(Exception {
                location: value.location,
                message: "Invalid negative value".into(),
            });
            0.0
        } else {
            number
        }
    }
}

pub fn create_presentation_from_ast(
    ast: BoundAst,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let mut evaluator = Evaluator::new();

    for module in &context.modules.modules {
        evaluator.set_variable(
            module.read().unwrap().name,
            Value {
                value: value::Value::Module(module.clone()),
                location: Location::zero(),
            },
        );
    }

    for statement in ast.statements {
        evaluate_statement(statement, &mut evaluator, context)?;
        if let Some(exception) = evaluator.exception.take() {
            exception.print(&context.loaded_files);
            break;
        }
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
        BoundNodeKind::Empty(()) => Ok(()),
        BoundNodeKind::Error(BoundError) => unreachable!("Errors should create errors!"),
        BoundNodeKind::StylingStatement(styling_statement) => {
            evaluate_styling_statement(styling_statement, statement.location, evaluator, context)
        }
        BoundNodeKind::SlideStatement(slide_statement) => {
            evaluate_slide_statement(slide_statement, evaluator, context)
        }
        BoundNodeKind::GlobalStatement(global_statement) => {
            evaluate_global_statement(global_statement, evaluator, context)
        }
        BoundNodeKind::ElementStatement(element_statement) => {
            evaluate_element_statement(element_statement, statement.location, evaluator, context)
        }
        BoundNodeKind::TemplateStatement(template_statement) => {
            evaluate_template_statement(template_statement, statement.location, evaluator, context)
        }
        BoundNodeKind::ImportStatement(import_statement) => {
            evaluate_import_statement(import_statement, evaluator, context)
        }
        err => unreachable!("No Top Level Statement: {err:?}"),
    }
}

fn evaluate_import_statement(
    import_statement: std::path::PathBuf,
    _evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let path_extensions = import_statement.to_str().unwrap().split('.').rev();
    enum State {
        Unknown,
        HtmlUnknown,
        HtmlHead,
        JavascriptUnknown,
        JavascriptInit,
    }

    impl State {
        pub fn is_finished(&self) -> bool {
            matches!(self, Self::HtmlHead | Self::JavascriptInit)
        }
    }
    let mut state = State::Unknown;
    for extension in path_extensions {
        match extension {
            "html" => state = State::HtmlUnknown,
            "head" => state = State::HtmlHead,
            "js" => state = State::JavascriptUnknown,
            "init" => state = State::JavascriptInit,
            missing => unreachable!("Missing {missing}"),
        }
        if state.is_finished() {
            break;
        }
    }
    let placement = match state {
        State::HtmlHead => FilePlacement::HtmlHead,
        State::JavascriptInit => FilePlacement::JavascriptInit,
        State::Unknown | State::HtmlUnknown | State::JavascriptUnknown => unreachable!(),
    };
    context.presentation.write().unwrap().add_extern_text(
        placement,
        slides_rs_core::ExternText::File(import_statement),
    )?;
    Ok(())
}

fn evaluate_element_statement(
    element_statement: super::binder::ElementStatement,
    location: Location,
    evaluator: &mut Evaluator,
    _context: &mut Context,
) -> slides_rs_core::Result<()> {
    let parameters = element_statement.parameters;
    evaluator.set_variable(
        element_statement.name,
        Value {
            value: value::Value::UserFunction(value::UserFunctionValue {
                has_implicit_slide_parameter: false,
                parameters,
                body: element_statement.body,
                return_type: element_statement.type_,
            }),
            location,
        },
    );
    Ok(())
}

fn evaluate_template_statement(
    template_statement: super::binder::TemplateStatement,
    location: Location,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let parameters = template_statement.parameters;
    let template = value::UserFunctionValue {
        has_implicit_slide_parameter: true,
        parameters,
        body: template_statement.body,
        return_type: TypeId::VOID,
    };
    if context
        .string_interner
        .resolve_variable(template_statement.name)
        == "default"
    {
        evaluator.default_template = Some(template);
        // TODO: Apply to previous slides here?
    } else {
        evaluator.set_variable(
            template_statement.name,
            Value {
                value: value::Value::UserFunction(template),
                location,
            },
        );
    }
    Ok(())
}

fn evaluate_slide_statement(
    slide_statement: super::binder::SlideStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let slide_count = context.presentation.read().unwrap().slide_count();
    let slide = Slide::new(slide_count).with_name(
        context
            .string_interner
            .resolve_variable(slide_statement.name),
    );
    evaluator.slide = Some(slide);
    slide::evaluate_to_slide(slide_statement.body, evaluator, context)?;
    if let Some(default_template) = &evaluator.default_template
        && !evaluator
            .slide
            .as_ref()
            .expect("should exist")
            .skips_default_template
    {
        slide::evaluate_expression(
            BoundNode::fake_function_call(default_template.clone(), vec![]),
            evaluator,
            context,
        );
    }
    let slide = evaluator
        .slide
        .take()
        .expect("Should still be set after evaluate_to_slide");

    context.presentation.write().unwrap().add_slide(slide);
    Ok(())
}

fn evaluate_global_statement(
    global_statement: super::binder::GlobalStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    for statement in global_statement.body {
        slide::evaluate_statement(statement, evaluator, context)?;
        if let Some(exception) = evaluator.exception.take() {
            exception.print(&context.loaded_files);
            return Ok(());
        }
    }
    Ok(())
}

fn evaluate_styling_statement(
    styling_statement: super::binder::StylingStatement,
    location: Location,
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
    evaluator.push_scope();
    let name = context.string_interner.create_or_get_variable("text");
    if styling_statement.type_ == StylingType::Label {
        evaluator.set_variable(
            name,
            Value {
                value: value::Value::TextStyling(Arc::new(RwLock::new(TextStyling::default()))),
                location,
            },
        );
    }
    style::evaluate_to_styling(styling_statement.body, evaluator, context);
    let mut styling = evaluator.styling.take().expect("styling");
    if let Some(value) = evaluator.try_get_variable(name) {
        styling
            .as_label_mut()
            .set_text_styling(value.value.clone().as_text_styling().get_cloned().unwrap());
    }
    evaluator.drop_scope();
    let reference = context
        .presentation
        .write()
        .unwrap()
        .add_dynamic_styling(styling);
    evaluator.set_variable(
        styling_statement.name,
        Value {
            value: value::Value::StyleReference(reference),
            location,
        },
    );
    Ok(())
}
