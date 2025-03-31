use std::{collections::HashMap, path::PathBuf};

use slides_rs_core::{Background, Color, Label, Slide, WebRenderable};
use string_interner::symbol::SymbolUsize;

use crate::Context;
use crate::compiler::binder::{self, BoundNode, BoundNodeKind, Value, typing::Type};

use super::Evaluator;

pub fn evaluate_to_slide(
    mut slide: Slide,
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<Slide> {
    evaluator.push_scope();
    evaluator.set_variable(
        "background".into(),
        Value::Background(Background::Unspecified),
    );
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
            evaluate_expression(statement, slide, evaluator, context);
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
    let value = evaluate_expression(*variable_declaration.value, slide, evaluator, context);
    evaluator.set_variable(name, value);
    Ok(())
}

fn evaluate_assignment(
    assignment_statement: crate::compiler::binder::AssignmentStatement,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let value = evaluate_expression(*assignment_statement.value, slide, evaluator, context);
    let target = resolve_assignment_target(*assignment_statement.lhs, evaluator, context);
    assign(slide, value, target, context);
    Ok(())
}

fn assign(slide: &mut Slide, value: Value, target: AssignmentTarget<'_>, context: &mut Context) {
    match target {
        AssignmentTarget::Variable(target, name) => match name.as_str() {
            "background" => {
                slide.styling_mut().set_background(value.into_background());
            }
            _ => {
                *target = value;
            }
        },
        AssignmentTarget::Member(base, member) => match context.string_interner.resolve(member) {
            "text_color" => {
                base.as_mut_label()
                    .element_styling_mut()
                    .set_text_color(value.into_color());
            }
            "object_fit" => {
                base.as_mut_image()
                    .element_styling_mut()
                    .set_object_fit(value.into_object_fit());
            }
            "valign" => {
                base.as_mut_image()
                    .positioning_mut()
                    .set_vertical_alignment(value.into_vertical_alignment());
            }
            "halign" => {
                base.as_mut_image()
                    .positioning_mut()
                    .set_horizontal_alignment(value.into_horizontal_alignment());
            }
            "background" => {
                base.as_mut_label()
                    .element_styling_mut()
                    .set_background(value.into_background());
            }
            err => todo!("Handle member {err}",),
        },
    }
}

fn evaluate_expression(
    expression: BoundNode,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> crate::compiler::binder::Value {
    match expression.kind {
        BoundNodeKind::FunctionCall(function_call) => {
            evaluate_function_call(function_call, slide, evaluator, context)
        }
        BoundNodeKind::VariableReference(variable) => todo!(),
        BoundNodeKind::Literal(value) => value,
        BoundNodeKind::Dict(dict) => evaluate_dict(dict, slide, evaluator, context),
        BoundNodeKind::MemberAccess(member_access) => {
            evaluate_member_access(member_access, evaluator, context)
        }
        BoundNodeKind::Conversion(conversion) => {
            evaluate_conversion(conversion, slide, evaluator, context)
        }
        BoundNodeKind::PostInitialization(post_initialization) => {
            evaluate_post_initialization(post_initialization, slide, evaluator, context)
        }
        _ => unreachable!("Only statements can be evaluated!"),
    }
}

fn evaluate_dict(
    dict: Vec<(String, BoundNode)>,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let mut result = HashMap::new();
    for (member, node) in dict {
        let value = evaluate_expression(node, slide, evaluator, context);
        result.insert(member, value);
    }
    result.into()
}

fn evaluate_post_initialization(
    post_initialization: binder::PostInitialization,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let mut base = evaluate_expression(*post_initialization.base, slide, evaluator, context);
    let dict = evaluate_expression(*post_initialization.dict, slide, evaluator, context);
    let dict = dict.into_dict();
    // let value = evaluate_expression_mut(*member_access.base, evaluator, context);

    for (member, value) in dict {
        let member = context.string_interner.create_or_get(&member);
        assign(
            slide,
            value,
            AssignmentTarget::Member(&mut base, member),
            context,
        );
    }
    base
}

fn evaluate_member_access(
    member_access: binder::MemberAccess,
    _evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    if let Some((enum_type, _)) = context
        .type_interner
        .resolve(member_access.base.type_)
        .unwrap_or(&Type::Error)
        .try_as_enum_ref()
    {
        let variant = context.string_interner.resolve(member_access.member);
        match &**enum_type {
            &Type::ObjectFit => Value::ObjectFit(variant.parse().expect("Valid variant")),
            &Type::HAlign => Value::HorizontalAlignment(variant.parse().expect("Valid variant")),
            &Type::VAlign => Value::VerticalAlignment(variant.parse().expect("Valid variant")),
            _ => unreachable!("Type {enum_type:?} is not an enum!"),
        }
    } else {
        todo!()
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
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let arguments: Vec<Value> = function_call
        .arguments
        .into_iter()
        .map(|a| evaluate_expression(a, slide, evaluator, context))
        .collect();
    let function_name = extract_function_name(*function_call.base, context);
    (binder::globals::FUNCTIONS
        .iter()
        .find(|f| f.name == function_name.as_str())
        .expect("Unknown Function")
        .call)(arguments)
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
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let base = evaluate_expression(*conversion.base, slide, evaluator, context);
    match context
        .type_interner
        .resolve(conversion.target)
        .unwrap_or(&Type::Error)
    {
        Type::Background => match base {
            Value::Color(color) => Value::Background(Background::Color(color)),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Color => match base {
            Value::String(text) => Value::Color(Color::from_css(&text)),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Path => match base {
            Value::String(text) => Value::Path(PathBuf::from(text)),
            _ => unreachable!("Impossible converion!"),
        },
        Type::Label => match base {
            Value::String(text) => Value::Label(Label::new(text)),
            _ => unreachable!("Impossible conversion!"),
        },
        _ => todo!(),
    }
}

enum AssignmentTarget<'a> {
    Variable(&'a mut Value, String),
    Member(&'a mut Value, SymbolUsize),
}

fn resolve_assignment_target<'a, 'b>(
    lhs: BoundNode,
    evaluator: &'a mut Evaluator,
    context: &'b mut Context,
) -> AssignmentTarget<'a> {
    match lhs.kind {
        BoundNodeKind::VariableReference(variable) => {
            let name = context.string_interner.resolve_variable(variable.id);
            dbg!(name);
            AssignmentTarget::Variable(evaluator.get_variable_mut(name), name.into())
        }
        BoundNodeKind::MemberAccess(member_access) => {
            let value = evaluate_expression_mut(*member_access.base, evaluator, context);
            AssignmentTarget::Member(value, member_access.member)
        }
        err => unreachable!("Cannot assign to {err:?}"),
    }
}
