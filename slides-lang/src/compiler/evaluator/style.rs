use slides_rs_core::{ElementStyling, ToCss};

use super::Evaluator;
use crate::Context;
use crate::compiler::binder::BoundNode;

pub fn evaluate_to_styling<S: ToCss + 'static>(
    init_value: ElementStyling<S>,
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::ElementStyling<S> {
    init_value
}
