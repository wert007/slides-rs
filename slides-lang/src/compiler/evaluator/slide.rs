use std::cell::RefCell;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use slides_rs_core::{
    Background, Color, CustomElement, Element, ElementStyling, Label, Thickness, WebRenderable,
};
use string_interner::symbol::SymbolUsize;

use crate::Context;
use crate::compiler::binder::{self, BoundNode, BoundNodeKind, typing::Type};

use super::Evaluator;
use super::value::{UserFunctionValue, Value};

pub fn evaluate_to_slide(
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    evaluator.push_scope();
    evaluator.set_variable(
        context.string_interner.create_or_get_variable("background"),
        Value::Background(Background::Unspecified),
    );
    for statement in body {
        evaluate_statement(statement, evaluator, context)?;
    }

    let scope = evaluator.drop_scope();
    let mut slide = evaluator.slide.take().expect("There should be a slide!");
    for (name, value) in scope.variables() {
        let name = context.string_interner.resolve_variable(name);
        match name {
            "background" => {
                slide.styling_mut().set_background(value.into_background());
                continue;
            }
            "text_color" => {
                slide.styling_mut().set_text_color(value.into_color());
                continue;
            }
            "padding" => {
                slide
                    .styling_mut()
                    .base_mut()
                    .set_padding(value.into_thickness());
                continue;
            }
            _ => {}
        }
        match value {
            Value::Label(label) => {
                let mut label = Arc::unwrap_or_clone(label).into_inner();
                label.set_id(name.into());
                label.set_z_index(slide.next_z_index());
                slide = slide.add_label(label);
            }
            Value::Image(image) => {
                let mut image = Arc::unwrap_or_clone(image).into_inner();
                image.set_id(name.into());
                image.set_z_index(slide.next_z_index());
                slide = slide.add_image(image);
            }
            Value::CustomElement(custom_element) => {
                let mut custom_element = Arc::unwrap_or_clone(custom_element).into_inner();
                custom_element.set_id(name.into());
                custom_element.set_z_index(slide.next_z_index());
                slide = slide.add_custom_element(custom_element);
            }

            _ => {}
        }
    }
    evaluator.slide = Some(slide);
    Ok(())
}

fn evaluate_statement(
    statement: BoundNode,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    match statement.kind {
        BoundNodeKind::AssignmentStatement(assignment_statement) => {
            evaluate_assignment(assignment_statement, evaluator, context)
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
            evaluate_variable_declaration(variable_declaration, evaluator, context)
        }
        _ => unreachable!("Internal Compiler Error"),
    }
}

fn evaluate_variable_declaration(
    variable_declaration: crate::compiler::binder::VariableDeclaration,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let value = evaluate_expression(*variable_declaration.value, evaluator, context);
    evaluator.set_variable(variable_declaration.variable, value);
    Ok(())
}

fn evaluate_assignment(
    assignment_statement: crate::compiler::binder::AssignmentStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::Result<()> {
    let value = evaluate_expression(*assignment_statement.value, evaluator, context);
    assign_to(*assignment_statement.lhs, value, evaluator, context);
    Ok(())
}

fn assign_to(node: BoundNode, value: Value, evaluator: &mut Evaluator, context: &mut Context) {
    match node.kind {
        BoundNodeKind::VariableReference(variable) => {
            evaluator.set_variable(variable.id, value);
        }
        BoundNodeKind::MemberAccess(member_access) => {
            let member = member_access.member;
            assign_member(*member_access.base, member, value, evaluator, context);
        }
        BoundNodeKind::Conversion(_conversion) => todo!(),
        _ => {
            unreachable!("Not assignable!")
        }
    }
}

fn assign_member(
    base: BoundNode,
    member: SymbolUsize,
    value: Value,
    evaluator: &mut Evaluator,
    context: &mut Context,
) {
    match base.kind {
        BoundNodeKind::Conversion(conversion) => {
            // TODO: Honour conversion!
            assign_member(*conversion.base, member, value, evaluator, context);
        }
        BoundNodeKind::VariableReference(variable) => {
            let base = evaluator.get_variable_mut(variable.id);
            let base_type = base.infer_type();
            match base_type {
                Type::Element | Type::Label | Type::Image | Type::CustomElement(_) => {
                    assign_to_slide_type(base_type, base, member, value, context);
                }
                missing => unreachable!("Missing {missing:?}"),
            }
        }
        missing => unreachable!("Missing {missing:?}"),
    }
}

pub(super) fn evaluate_expression(
    expression: BoundNode,
    // slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    match expression.kind {
        BoundNodeKind::FunctionCall(function_call) => {
            evaluate_function_call(function_call, evaluator, context)
        }
        BoundNodeKind::VariableReference(variable) => evaluator.get_variable(variable.id).clone(),
        BoundNodeKind::Literal(value) => value,
        BoundNodeKind::Dict(dict) => evaluate_dict(dict, evaluator, context),
        BoundNodeKind::MemberAccess(member_access) => {
            evaluate_member_access(member_access, evaluator, context)
        }
        BoundNodeKind::Conversion(conversion) => {
            evaluate_conversion(conversion, evaluator, context)
        }
        BoundNodeKind::PostInitialization(post_initialization) => {
            evaluate_post_initialization(post_initialization, evaluator, context)
        }
        BoundNodeKind::Array(array) => evaluate_array(array, evaluator, context),
        BoundNodeKind::Binary(binary) => evaluate_binary(binary, evaluator, context),
        _ => unreachable!("Only expressions can be evaluated!"),
    }
}

fn evaluate_binary(
    binary: binder::Binary,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let lhs = evaluate_expression(*binary.lhs, evaluator, context);
    let rhs = evaluate_expression(*binary.rhs, evaluator, context);
    binary.operator.execute(lhs, rhs)
}

fn evaluate_array(
    array: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let mut values = Vec::with_capacity(array.len());
    for value in array {
        let value = evaluate_expression(value, evaluator, context);
        values.push(value);
    }
    Value::Array(values)
}

fn evaluate_dict(
    dict: Vec<(String, BoundNode)>,
    // slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let mut result = HashMap::new();
    for (member, node) in dict {
        let value = evaluate_expression(node, evaluator, context);
        result.insert(member, value);
    }
    result.into()
}

fn evaluate_post_initialization(
    post_initialization: binder::PostInitialization,
    // slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let base_type = post_initialization.base.type_;
    let mut base = evaluate_expression(*post_initialization.base, evaluator, context);
    let dict = evaluate_expression(*post_initialization.dict, evaluator, context);
    let dict = dict.into_dict();
    // let value = evaluate_expression_mut(*member_access.base, evaluator, context);

    for (member, value) in dict {
        let member = context.string_interner.create_or_get(&member);
        let base_type = context.type_interner.resolve(base_type).unwrap().clone();
        match base_type {
            Type::Element | Type::Label | Type::CustomElement(_) | Type::Image => {
                assign_to_slide_type(base_type, &mut base, member, value, context)
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
    _base_type: Type,
    base: &mut Value,
    member: SymbolUsize,
    value: Value,
    context: &mut Context,
) {
    let member = context.string_interner.resolve(member);
    match member {
        "width" => {
            base.as_mut_base_element()
                .set_width(value.into_style_unit());
        }
        "height" => {
            base.as_mut_base_element()
                .set_height(value.into_style_unit());
        }
        "z_index" => {
            base.as_mut_base_element()
                .set_z_index(value.into_integer() as _);
        }
        "valign" => {
            base.as_mut_base_element()
                .set_vertical_alignment(value.into_vertical_alignment());
        }
        "halign" => {
            base.as_mut_base_element()
                .set_horizontal_alignment(value.into_horizontal_alignment());
        }
        "margin" => {
            base.as_mut_base_element()
                .set_margin(value.into_thickness());
        }
        "padding" => {
            base.as_mut_base_element()
                .set_padding(value.into_thickness());
        }
        "background" => {
            base.as_mut_base_element()
                .set_background(value.into_background());
        }
        "filter" => {
            base.as_mut_base_element().set_filter(value.into_filter());
        }
        "styles" => {
            for value in value.into_array() {
                base.as_mut_base_element()
                    .add_styling_reference(value.into_style_reference())
            }
        }
        "object_fit" => {
            base.as_mut_image()
                .borrow_mut()
                .element_styling_mut()
                .set_object_fit(value.into_object_fit());
        }
        "text_color" => {
            base.as_mut_label()
                .borrow_mut()
                .element_styling_mut()
                .set_text_color(value.into_color());
        }
        "text_align" => {
            base.as_mut_label()
                .borrow_mut()
                .element_styling_mut()
                .set_text_align(value.into_text_align());
        }
        "font_size" => {
            base.as_mut_label()
                .borrow_mut()
                .element_styling_mut()
                .set_font_size(value.into_float());
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
    // slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let arguments: Vec<Value> = function_call
        .arguments
        .into_iter()
        .map(|a| evaluate_expression(a, evaluator, context))
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
                evaluator,
                context,
            )
        }
    }
}

fn evaluate_user_function(
    user_function: UserFunctionValue,
    arguments: Vec<Value>,
    // slide: &mut Slide,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let implicit_fields: &[(crate::VariableId, Value)] =
        if user_function.has_implicit_slide_parameter {
            &[(
                context
                    .string_interner
                    .create_or_get_variable("slide_index"),
                Value::Integer(evaluator.slide.as_ref().unwrap().index as i64),
            )]
        } else {
            &[]
        };
    let scope = evaluator.push_scope();
    for (id, value) in implicit_fields {
        scope.set_variable(*id, value.clone());
    }
    for (parameter, value) in user_function.parameters.into_iter().zip(
        arguments
            .into_iter()
            .map(|v| Some(v))
            .chain(std::iter::repeat(None)),
    ) {
        let value = value.or_else(|| parameter.value).unwrap();
        scope.set_variable(parameter.id, value);
    }
    for statement in user_function.body {
        evaluate_statement(statement, evaluator, context).unwrap();
    }

    let scope = evaluator.drop_scope();
    let type_name = context
        .type_interner
        .resolve(user_function.return_type)
        .unwrap()
        .try_as_custom_element_ref()
        .cloned()
        .unwrap_or_default();
    let mut custom_element = CustomElement::new(type_name, Vec::new());
    let mut slide = evaluator.slide.take().expect("slide");
    let styling = if user_function.has_implicit_slide_parameter {
        slide.styling_mut().base_mut()
    } else {
        custom_element.element_styling_mut().base_mut()
    };
    let mut elements: Vec<Element> = Vec::new();
    for (name, value) in scope.variables() {
        match context.string_interner.resolve_variable(name) {
            "halign" => {
                styling.set_horizontal_alignment(value.into_horizontal_alignment());
                continue;
            }
            "valign" => {
                styling.set_vertical_alignment(value.into_vertical_alignment());
                continue;
            }
            _ => {}
        }
        match value {
            Value::Label(label) => {
                let mut label = Arc::unwrap_or_clone(label).into_inner();
                let name = context.string_interner.resolve_variable(name);
                label.set_id(name.into());
                elements.push(label.into());
            }
            Value::Image(image) => {
                let mut image = Arc::unwrap_or_clone(image).into_inner();
                let name = context.string_interner.resolve_variable(name);
                image.set_id(name.into());
                elements.push(image.into());
            }
            Value::CustomElement(element) => {
                let mut element = Arc::unwrap_or_clone(element).into_inner();
                let name = context.string_interner.resolve_variable(name);
                element.set_id(name.into());
                elements.push(element.into());
            }

            _ => {}
        }
    }

    let result = if user_function.has_implicit_slide_parameter {
        for element in elements {
            slide = slide.add_element(element);
        }
        Value::Void(())
    } else {
        let custom_element = custom_element.with_elements(elements);
        Value::CustomElement(Arc::new(RefCell::new(custom_element)))
    };
    evaluator.slide = Some(slide);
    result
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
            Value::String(text) => Value::Label(Arc::new(RefCell::new(Label::new(text)))),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Element => match base {
            Value::Label(_label) => todo!(),
            Value::Image(_image) => todo!(),
            Value::CustomElement(_custom_element) => todo!(),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Thickness => match base {
            Value::Dict(entries) => {
                let mut thickness = Thickness::default();
                for (name, value) in entries {
                    match name.as_str() {
                        "top" => thickness.top = value.into_style_unit(),
                        "left" => thickness.left = value.into_style_unit(),
                        "bottom" => thickness.bottom = value.into_style_unit(),
                        "right" => thickness.right = value.into_style_unit(),
                        _ => unreachable!("Impossible conversion"),
                    }
                }
                thickness.into()
            }
            _ => unreachable!("Impossible conversion"),
        },
        unknown => todo!("{unknown:?}"),
    }
}
