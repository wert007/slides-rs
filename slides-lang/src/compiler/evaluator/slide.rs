use std::{collections::HashMap, path::PathBuf};

use slides_rs_core::{Background, Color, CustomElement, Label, Slide, WebRenderable};
use string_interner::symbol::SymbolUsize;

use crate::compiler::binder::{self, BoundNode, BoundNodeKind, Value, typing::Type};
use crate::{Context, VariableId};

use super::Evaluator;

pub fn evaluate_to_slide(
    mut slide: Slide,
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<Slide> {
    evaluator.push_scope();
    evaluator.set_variable(
        context.string_interner.create_or_get_variable("background"),
        Value::Background(Background::Unspecified),
    );
    for statement in body {
        evaluate_statement(statement, &mut slide, evaluator, context)?;
    }

    let scope = evaluator.drop_scope();
    for (name, value) in scope.variables {
        match value {
            Value::Label(mut label) => {
                let name = context.string_interner.resolve_variable(name);
                label.set_id(name.into());
                slide = slide.add_label(label);
            }
            Value::Image(mut image) => {
                let name = context.string_interner.resolve_variable(name);
                image.set_id(name.into());
                slide = slide.add_image(image);
            }
            Value::CustomElement(mut custom_element) => {
                let name = context.string_interner.resolve_variable(name);
                custom_element.set_id(name.into());
                slide = slide.add_custom_element(custom_element);
            }

            _ => {}
        }
    }
    Ok(slide)
}

fn evaluate_statement(
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
    let value = evaluate_expression(*variable_declaration.value, slide, evaluator, context);
    // dbg!(&value);
    evaluator.set_variable(variable_declaration.variable, value);
    dbg!(evaluator.get_variable(variable_declaration.variable));
    Ok(())
}

fn evaluate_assignment(
    assignment_statement: crate::compiler::binder::AssignmentStatement,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let value = evaluate_expression(*assignment_statement.value, slide, evaluator, context);
    assign_to(*assignment_statement.lhs, slide, value, evaluator, context);
    Ok(())
}

fn assign_to(
    node: BoundNode,
    slide: &mut Slide,
    value: Value,
    evaluator: &mut Evaluator,
    context: &mut Context,
) {
    match node.kind {
        BoundNodeKind::VariableReference(variable) => {
            evaluator.set_variable(variable.id, value);
        }
        BoundNodeKind::MemberAccess(member_access) => {
            let member = member_access.member;
            assign_member(
                *member_access.base,
                slide,
                member,
                value,
                evaluator,
                context,
            );
        }
        BoundNodeKind::Conversion(conversion) => todo!(),
        _ => {
            unreachable!("Not assignable!")
        }
    }
}

fn assign_member(
    base: BoundNode,
    slide: &mut Slide,
    member: SymbolUsize,
    value: Value,
    evaluator: &mut Evaluator,
    context: &mut Context,
) {
    match base.kind {
        BoundNodeKind::Conversion(conversion) => {
            // TODO: Honour conversion!
            assign_member(*conversion.base, slide, member, value, evaluator, context);
        }
        BoundNodeKind::VariableReference(variable) => {
            let base = evaluator.get_variable_mut(variable.id);
            let base_type = base.infer_type();
            match base_type {
                Type::Element | Type::Label | Type::Image | Type::CustomElement(_) => {
                    assign_to_slide_type(base_type, base, member, value, slide, context);
                }
                missing => unreachable!("Missing {missing:?}"),
            }
        }
        missing => unreachable!("Missing {missing:?}"),
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
        BoundNodeKind::VariableReference(variable) => evaluator.get_variable(variable.id).clone(),
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
    let base_type = post_initialization.base.type_;
    let mut base = evaluate_expression(*post_initialization.base, slide, evaluator, context);
    let dict = evaluate_expression(*post_initialization.dict, slide, evaluator, context);
    let dict = dict.into_dict();
    // let value = evaluate_expression_mut(*member_access.base, evaluator, context);

    for (member, value) in dict {
        let member = context.string_interner.create_or_get(&member);
        let base_type = context.type_interner.resolve(base_type).unwrap().clone();
        match base_type {
            Type::Element | Type::Label | Type::CustomElement(_) | Type::Image => {
                assign_to_slide_type(base_type, &mut base, member, value, slide, context)
            }
            _ => {
                todo!();
            }
        }
        // assign_member(base, slide, member, value, evaluator, context);
        // assign_member(slide, value, member, &mut base, evaluator, context)
        // assign(
        //     slide,
        //     value,
        //     AssignmentTarget::Member(member),
        //     evaluator,
        //     context,
        // );
    }
    // let mut base = Value::Integer(0);
    // std::mem::swap(&mut base, evaluator.accumulator_mut());
    base
}

fn assign_to_slide_type(
    base_type: Type,
    base: &mut Value,
    member: SymbolUsize,
    value: Value,
    slide: &mut Slide,
    context: &mut Context,
) {
    let member = context.string_interner.resolve(member);
    match member {
        "valign" => {
            base.as_mut_base_element()
                .positioning_mut()
                .set_vertical_alignment(value.into_vertical_alignment());
        }
        "halign" => {
            base.as_mut_base_element()
                .positioning_mut()
                .set_horizontal_alignment(value.into_horizontal_alignment());
        }
        "background" => {
            base.as_mut_base_element()
                .element_styling_mut()
                .set_background(value.into_background());
        }
        "object_fit" => {
            base.as_mut_image()
                .element_styling_mut()
                .set_object_fit(value.into_object_fit());
        }
        "text_color" => {
            base.as_mut_label()
                .element_styling_mut()
                .set_text_color(value.into_color());
        }
        "text_align" => {
            base.as_mut_label()
                .element_styling_mut()
                .set_text_align(value.into_text_align());
        }
        missing => unreachable!("Missing Member {missing}"),
    }
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
            &Type::TextAlign => Value::TextAlign(variant.parse().expect("Valid variant")),
            _ => unreachable!("Type {enum_type:?} is not an enum!"),
        }
    } else {
        todo!()
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
    match binder::globals::FUNCTIONS
        .iter()
        .find(|f| f.name == function_name.as_str())
    {
        Some(it) => (it.call)(arguments),
        None => {
            let value = evaluator
                .get_variable_mut(
                    context
                        .string_interner
                        .create_or_get_variable(&function_name),
                )
                .clone();
            evaluate_user_function(
                value.as_user_function().clone(),
                arguments,
                slide,
                evaluator,
                context,
            )
        }
    }
}

fn evaluate_user_function(
    user_function: binder::UserFunctionValue,
    arguments: Vec<Value>,
    slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let scope = evaluator.push_scope();
    for (parameter, value) in user_function.parameters.into_iter().zip(arguments) {
        scope.set_variable(parameter, value);
    }
    for statement in user_function.body {
        evaluate_statement(statement, slide, evaluator, context).unwrap();
    }

    let scope = evaluator.drop_scope();
    let mut elements = Vec::new();
    for (name, value) in scope.variables {
        match value {
            Value::Label(mut label) => {
                let name = context.string_interner.resolve_variable(name);
                label.set_id(name.into());
                label.set_z_index(slide.next_z_index());
                elements.push(label.into());
            }
            Value::Image(mut image) => {
                let name = context.string_interner.resolve_variable(name);
                image.set_id(name.into());
                image.set_z_index(slide.next_z_index());
                elements.push(image.into());
            }
            Value::CustomElement(mut element) => {
                let name = context.string_interner.resolve_variable(name);
                element.set_id(name.into());
                element.set_z_index(slide.next_z_index());
                elements.push(element.into());
            }

            _ => {}
        }
    }

    let type_name = context
        .type_interner
        .resolve(user_function.return_type)
        .unwrap()
        .try_as_custom_element_ref()
        .unwrap();
    Value::CustomElement(CustomElement::new(type_name, elements))
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
        Type::Element => match base {
            Value::Label(label) => todo!(),
            Value::Image(image) => todo!(),
            Value::CustomElement(custom_element) => todo!(),
            _ => unreachable!("Impossible conversion!"),
        },
        unknown => todo!("{unknown:?}"),
    }
}
