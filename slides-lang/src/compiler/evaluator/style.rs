use slides_rs_core::{ElementStyling, ToCss};

use super::Evaluator;
use crate::compiler::binder::{BoundNode, BoundNodeKind, Value};
use crate::{Context, VariableId};

pub fn evaluate_to_styling(body: Vec<BoundNode>, evaluator: &mut Evaluator, context: &mut Context) {
    for statement in body {
        evaluate_statement(statement, evaluator, context);
    }
}

fn evaluate_statement(statement: BoundNode, evaluator: &mut Evaluator, context: &mut Context) {
    match statement.kind {
        BoundNodeKind::AssignmentStatement(assignment_statement) => {
            evaluate_assignment_statement(assignment_statement, evaluator, context)
        }
        BoundNodeKind::FunctionCall(function_call) => todo!(),
        BoundNodeKind::VariableReference(variable) => todo!(),
        BoundNodeKind::Literal(value) => todo!(),
        BoundNodeKind::VariableDeclaration(variable_declaration) => {
            todo!()
        }
        BoundNodeKind::Dict(items) => todo!(),
        BoundNodeKind::MemberAccess(member_access) => todo!(),
        BoundNodeKind::Conversion(conversion) => todo!(),
        BoundNodeKind::PostInitialization(post_initialization) => todo!(),
        _ => unreachable!(),
    }
}

fn evaluate_assignment_statement(
    assignment_statement: crate::compiler::binder::AssignmentStatement,
    evaluator: &mut Evaluator,
    context: &mut Context,
) {
    let value: Value =
        super::slide::evaluate_expression(*assignment_statement.value, evaluator, context);
    assign_to(*assignment_statement.lhs, value, evaluator, context);
}

fn assign_to(lhs: BoundNode, value: Value, evaluator: &mut Evaluator, context: &mut Context) {
    match lhs.kind {
        BoundNodeKind::VariableReference(variable) => {
            assign_to_field(variable.id, value, evaluator, context);
        }
        BoundNodeKind::MemberAccess(member_access) => {
            todo!()
        }
        BoundNodeKind::Conversion(conversion) => todo!(),
        _ => unreachable!(),
    }
}

fn assign_to_field(
    name: VariableId,
    value: Value,
    evaluator: &mut Evaluator,
    context: &mut Context,
) {
    let styling = evaluator.styling.as_mut().expect("");
    match context.string_interner.resolve_variable(name) {
        "halign" => {
            styling
                .as_base_mut()
                .set_horizontal_alignment(value.into_horizontal_alignment());
        }
        "valign" => {
            styling
                .as_base_mut()
                .set_vertical_alignment(value.into_vertical_alignment());
        }
        "font" => {
            styling.as_label_mut().set_font(value.into_font());
        }
        "text_align" => {
            styling
                .as_label_mut()
                .set_text_align(value.into_text_align());
        }
        "font_size" => {
            styling
                .as_label_mut()
                .set_font_size(value.into_style_unit());
        }
        "text_color" => {
            styling.as_label_mut().set_text_color(value.into_color());
        }
        "background" => {
            styling
                .as_base_mut()
                .set_background(value.into_background());
        }
        "margin" => {
            styling.as_base_mut().set_margin(value.into_thickness());
        }
        "padding" => {
            styling.as_base_mut().set_padding(value.into_thickness());
        }
        name => unreachable!("UNknown {name}"),
    }
}
