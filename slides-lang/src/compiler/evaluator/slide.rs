use std::path::PathBuf;

use slides_rs_core::{Background, Color, Label, Slide, WebRenderable};
use string_interner::{Symbol, symbol::SymbolUsize};

use crate::compiler::{
    Context,
    binder::{self, BoundNode, BoundNodeKind, Type, Value},
};

use super::Evaluator;

pub fn evaluate_to_slide(
    mut slide: Slide,
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<Slide> {
    evaluator.push_scope();
    for statement in body {
        evalute_statement(statement, &mut slide, evaluator, context)?;
    }

    let scope = evaluator.drop_scope();
    for (name, value) in scope.variables {
        match value {
            Value::Label(mut label) => {
                label.set_id(name);
                slide = slide.add_label(label);
            }
            Value::Image(mut image) => {
                image.set_id(name);
                slide = slide.add_image(image);
            }

            _ => {}
        }
    }
    Ok(slide)
}

fn evalute_statement(
    statement: BoundNode,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    match statement.kind {
        BoundNodeKind::AssignmentStatement(assignment_statement) => {
            evaluate_assignment(assignment_statement, slide, evaluator, context)
        }
        BoundNodeKind::FunctionCall(_)
        | BoundNodeKind::VariableReference(_)
        | BoundNodeKind::Literal(_)
        | BoundNodeKind::Dict(_)
        | BoundNodeKind::MemberAccess(_)
        | BoundNodeKind::Conversion(_) => {
            evaluate_expression(statement, evaluator, context);
            Ok(())
        }
        BoundNodeKind::VariableDeclaration(variable_declaration) => {
            evaluate_variable_declaration(variable_declaration, slide, evaluator, context)
        }
        _ => unreachable!("Internal Compiler Error"),
    }
}

fn evaluate_variable_declaration(
    variable_declaration: crate::compiler::binder::VariableDeclaration,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let name = context
        .string_interner
        .resolve_variable(variable_declaration.variable)
        .to_string();
    let value = evaluate_expression(*variable_declaration.value, evaluator, context);
    evaluator.set_variable(name, value);
    Ok(())
}

fn evaluate_assignment(
    assignment_statement: crate::compiler::binder::AssignmentStatement,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let value = evaluate_expression(*assignment_statement.value, evaluator, context);
    let target = resolve_assignment_target(*assignment_statement.lhs, evaluator, context);
    match target {
        AssignmentTarget::Variable(name) => match name.as_str() {
            "background" => {
                slide.styling_mut().set_background(value.into_background());
            }
            _ => {
                evaluator.set_variable(name, value);
            }
        },
        AssignmentTarget::Member(base, member) => match context.string_interner.resolve(member) {
            "text_color" => {
                base.as_mut_label()
                    .element_styling_mut()
                    .set_text_color(value.into_color());
            }
            "background" => {
                base.as_mut_label()
                    .element_styling_mut()
                    .set_background(value.into_background());
            }
            err => todo!("Handle member {err}",),
        },
    }
    Ok(())
}

fn evaluate_expression(
    expression: BoundNode,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> crate::compiler::binder::Value {
    match expression.kind {
        BoundNodeKind::FunctionCall(function_call) => {
            evaluate_function_call(function_call, evaluator, context)
        }
        BoundNodeKind::VariableReference(variable) => todo!(),
        BoundNodeKind::Literal(value) => value,
        BoundNodeKind::Dict(items) => todo!(),
        BoundNodeKind::MemberAccess(member_access) => todo!(),
        BoundNodeKind::Conversion(conversion) => {
            evaluate_conversion(conversion, evaluator, context)
        }
        _ => unreachable!("Only statements can be evaluated!"),
    }
}

fn evaluate_expression_mut<'a>(
    expression: BoundNode,
    evaluator: &'a mut Evaluator,
    context: &mut Context,
) -> &'a mut crate::compiler::binder::Value {
    match expression.kind {
        BoundNodeKind::FunctionCall(function_call) => {
            todo!()
        }
        BoundNodeKind::VariableReference(variable) => {
            let name = context.string_interner.resolve_variable(variable.id);
            evaluator.get_variable_mut(name)
        }
        BoundNodeKind::Literal(_) => unreachable!("Can never be mutable!"),
        BoundNodeKind::Dict(items) => todo!(),
        BoundNodeKind::MemberAccess(member_access) => todo!(),
        BoundNodeKind::Conversion(conversion) => {
            todo!()
        }
        _ => unreachable!("Only statements can be evaluated!"),
    }
}

fn evaluate_function_call(
    function_call: crate::compiler::binder::FunctionCall,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let arguments: Vec<Value> = function_call
        .arguments
        .into_iter()
        .map(|a| evaluate_expression(a, evaluator, context))
        .collect();
    let function_name = extract_function_name(*function_call.base, context);
    (binder::globals::FUNCTIONS
        .iter()
        .find(|f| f.name == function_name.as_str())
        .expect("Unknown Function")
        .call)(arguments)
    // match function_name.as_str() {
    //     "rgb" => Value::Color(super::functions::rgb(
    //         *arguments[0].as_integer(),
    //         *arguments[1].as_integer(),
    //         *arguments[2].as_integer(),
    //     )),
    //     f => unreachable!("Unknown Function {f}!"),
    // }
}

fn extract_function_name(base: BoundNode, context: &mut Context) -> String {
    match base.kind {
        BoundNodeKind::VariableReference(variable) => context
            .string_interner
            .resolve_variable(variable.id)
            .to_string(),
        _ => todo!("Handle dynamic functions!"),
    }
}

fn evaluate_conversion(
    conversion: crate::compiler::binder::Conversion,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let base = evaluate_expression(*conversion.base, evaluator, context);
    match conversion.target {
        Type::Error => todo!(),
        Type::Void => todo!(),
        Type::Float => todo!(),
        Type::Integer => todo!(),
        Type::String => todo!(),
        Type::Dict => todo!(),
        Type::Styling => todo!(),
        Type::Background => match base {
            Value::Color(color) => Value::Background(Background::Color(color)),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Color => match base {
            Value::String(text) => Value::Color(Color::from_css(&text)),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::ObjectFit => todo!(),
        Type::Function(_) => todo!(),
        Type::Slide => todo!(),
        Type::Image => todo!(),
        Type::Path => match base {
            Value::String(text) => Value::Path(PathBuf::from(text)),
            _ => unreachable!("Impossible converion!"),
        },
        Type::Label => match base {
            Value::String(text) => Value::Label(Label::new(text)),
            _ => unreachable!("Impossible conversion!"),
        },
    }
}

enum AssignmentTarget<'a> {
    Variable(String),
    Member(&'a mut Value, SymbolUsize),
}

fn resolve_assignment_target<'a>(
    lhs: BoundNode,
    evaluator: &'a mut Evaluator,
    context: &mut Context,
) -> AssignmentTarget<'a> {
    match lhs.kind {
        BoundNodeKind::VariableReference(variable) => AssignmentTarget::Variable(
            context
                .string_interner
                .resolve_variable(variable.id)
                .to_owned(),
        ),
        BoundNodeKind::MemberAccess(member_access) => {
            let value = evaluate_expression_mut(*member_access.base, evaluator, context);
            AssignmentTarget::Member(value, member_access.member)
        }
        err => unreachable!("Cannot assign to {err:?}"),
    }
}
