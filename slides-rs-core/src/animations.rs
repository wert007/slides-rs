use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Trigger {
    StepReached(usize),
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub trigger: Trigger,
    pub value: AnimationValue,
}

#[derive(Debug, Clone)]
pub enum AnimationValue {
    ClassAddition(String),
    ClassRemoval(String),
    FieldChange {
        field_name: String,
        previous_value: String,
        new_value: String,
    },
}

#[derive(Debug, Clone)]
pub struct Animations {
    animations: HashMap<Trigger, Vec<AnimationValue>>,
}

impl Animations {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }
}
