use slides_rs_core::{ElementStyling, ToCss};

use crate::compiler::{Context, binder::BoundNode};

use super::Evaluator;

pub fn evaluate_to_styling<S: ToCss + 'static>(
    init_value: ElementStyling<S>,
    body: Vec<BoundNode>,
    evaluator: &mut Evaluator,
    context: &mut Context,
) -> slides_rs_core::ElementStyling<S> {
    init_value
}
