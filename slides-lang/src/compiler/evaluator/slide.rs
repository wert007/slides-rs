use std::cell::RefCell;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use slides_rs_core::{Background, Color, CustomElement, Element, Label, Thickness, WebRenderable};
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
            "steps" => {
                slide.set_step_count(value.into_integer() as usize);
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
        let Some(mut element) = value.try_convert_to_element() else {
            continue;
        };
        if element.parent().is_some() {
            // Is already displayed as part of another element.
            continue;
        }
        element.set_name(name.into());
        element.set_z_index(slide.next_z_index());
        slide = slide.add_element(element);
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
        | BoundNodeKind::PostInitialization(_)
        | BoundNodeKind::Literal(_)
        | BoundNodeKind::Dict(_)
        | BoundNodeKind::MemberAccess(_)
        | BoundNodeKind::Conversion(_) => {
            let value = evaluate_expression(statement, evaluator, context);
            if let Some(element) = value.try_convert_to_element() {
                if element.parent().is_none() {
                    evaluator.slide.as_mut().unwrap().add_element_ref(element);
                }
            }
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
                Type::Element
                | Type::Label
                | Type::Image
                | Type::CustomElement(_)
                | Type::GridEntry => {
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
        let base_type = context.type_interner.resolve(base_type).clone();
        match base_type {
            Type::Element
            | Type::Label
            | Type::CustomElement(_)
            | Type::Image
            | Type::Grid
            | Type::Flex => assign_to_slide_type(base_type, &mut base, member, value, context),
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
                .set_width(value.into_style_unit().into());
        }
        "height" => {
            base.as_mut_base_element()
                .set_height(value.into_style_unit().into());
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
        "animations" => {
            let animations = value
                .into_array()
                .into_iter()
                .map(|v| v.into_animation())
                .collect();
            base.as_mut_base_element().set_animations(animations);
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
        "column_span" => {
            base.as_grid_entry().borrow_mut().column_span = value.into_integer() as _;
        }
        "children" => {
            for element in value.into_array() {
                base.as_grid()
                    .borrow_mut()
                    .add_element(element.convert_to_element());
            }
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
    execute_function(*function_call.base, arguments, evaluator, context)
}

fn execute_function(
    base: BoundNode,
    arguments: Vec<Value>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    match base.kind {
        BoundNodeKind::VariableReference(variable) => {
            let name = context
                .string_interner
                .resolve_variable(variable.id)
                .to_owned();
            execute_named_function(name, arguments, evaluator, context)
        }
        BoundNodeKind::MemberAccess(member_access) => {
            let base = evaluate_expression(*member_access.base, evaluator, context);
            let name = context
                .string_interner
                .resolve(member_access.member)
                .to_owned();
            execute_member_function(base, name, arguments, evaluator, context)
        }
        _ => todo!("Add function handling!"),
    }
}

fn execute_member_function(
    base: Value,
    name: String,
    mut arguments: Vec<Value>,
    _evaluator: &mut Evaluator,
    _context: &mut Context,
) -> Value {
    match base {
        Value::Grid(base) => match name.as_str() {
            "add" => {
                let element = match arguments.swap_remove(0) {
                    Value::Label(it) => {
                        it.borrow_mut().set_parent(base.borrow().id());
                        Arc::unwrap_or_clone(it).into_inner().into()
                    }
                    Value::Grid(it) => {
                        it.borrow_mut().set_parent(base.borrow().id());
                        Arc::unwrap_or_clone(it).into_inner().into()
                    }
                    Value::Flex(it) => {
                        it.borrow_mut().set_parent(base.borrow().id());
                        Arc::unwrap_or_clone(it).into_inner().into()
                    }
                    Value::Image(it) => {
                        it.borrow_mut().set_parent(base.borrow().id());
                        Arc::unwrap_or_clone(it).into_inner().into()
                    }
                    Value::CustomElement(it) => {
                        it.borrow_mut().set_parent(base.borrow().id());
                        Arc::unwrap_or_clone(it).into_inner().into()
                    }
                    Value::Element(mut it) => {
                        it.set_parent(base.borrow().id());
                        it
                    }
                    _ => unreachable!("Invalid argument!"),
                };
                Value::GridEntry(base.borrow_mut().add_element(element))
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}

fn execute_named_function(
    name: String,
    arguments: Vec<Value>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    match binder::globals::FUNCTIONS
        .iter()
        .find(|f| f.name == name.as_str())
    {
        Some(it) => (it.call)(arguments),
        None => {
            let value = evaluator
                .get_variable_mut(context.string_interner.create_or_get_variable(&name))
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
        let Some(mut element) = value.try_convert_to_element() else {
            continue;
        };
        let name = context.string_interner.resolve_variable(name);
        element.set_name(name.into());
        elements.push(element);
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

fn evaluate_conversion(
    conversion: crate::compiler::binder::Conversion,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> Value {
    let base = evaluate_expression(*conversion.base, evaluator, context);
    match context.type_interner.resolve(conversion.target) {
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
            value @ (Value::Label(_)
            | Value::Image(_)
            | Value::CustomElement(_)
            | Value::Grid(_)
            | Value::Flex(_)) => value,
            // Value::Label(label) => Value::Element(Arc::new(RefCell::new(Element::Label(
            //     Arc::unwrap_or_clone(label).into_inner(),
            // )))),
            // Value::Image(image) => Value::Element(Arc::new(RefCell::new(Element::Image(
            //     Arc::unwrap_or_clone(image).into_inner(),
            // )))),
            // Value::CustomElement(custom_element) => Value::Element(Arc::new(RefCell::new(
            //     Element::CustomElement(Arc::unwrap_or_clone(custom_element).into_inner()),
            // ))),
            // Value::Grid(grid) => Value::Element(Arc::new(RefCell::new(Element::Grid(
            //     Arc::unwrap_or_clone(grid).into_inner(),
            // )))),
            _ => unreachable!("Impossible conversion!"),
        },
        Type::Thickness => match base {
            Value::Dict(entries) => {
                let mut thickness = Thickness::default();
                for (name, value) in entries {
                    match name.as_str() {
                        "top" => thickness.top = value.into_style_unit().into(),
                        "left" => thickness.left = value.into_style_unit().into(),
                        "bottom" => thickness.bottom = value.into_style_unit().into(),
                        "right" => thickness.right = value.into_style_unit().into(),
                        _ => unreachable!("Impossible conversion"),
                    }
                }
                thickness.into()
            }
            _ => unreachable!("Impossible conversion"),
        },
        Type::String => match base {
            Value::Float(x) => Value::String(x.to_string()),
            Value::Integer(x) => Value::String(x.to_string()),
            Value::String(x) => Value::String(x),
            Value::Path(x) => Value::String(x.to_string_lossy().into_owned()),
            _ => unreachable!("Impossible conversion"),
        },
        unknown => todo!("{unknown:?}"),
    }
}
