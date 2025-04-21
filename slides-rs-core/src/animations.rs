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
    animations: Vec<Animation>,
}

impl Animations {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add_animations(&mut self, animations: &[Animation]) {
        self.animations.extend_from_slice(animations);
    }

    pub(crate) fn get_initial_classes(&self) -> String {
        self.animations
            .iter()
            .filter_map(|a| match &a.value {
                crate::animations::AnimationValue::ClassAddition(_) => None,
                crate::animations::AnimationValue::ClassRemoval(class) => Some(class.clone()),
                crate::animations::AnimationValue::FieldChange { .. } => None,
            })
            .intersperse(" ".into())
            .collect()
    }

    pub(crate) fn emit_to_javascript<W: std::io::Write>(
        &self,
        w: &mut W,
        ctx: crate::WebRenderableContext,
        id: &str,
    ) -> std::io::Result<()> {
        for animation in &self.animations {
            match animation.trigger {
                crate::animations::Trigger::StepReached(number) => {
                    writeln!(
                        w,
                        "stepReached.push({{\n slideId: '{}',\n step: {number},\n trigger: () => {{",
                        ctx.slide_name
                    )?;
                    match &animation.value {
                        crate::animations::AnimationValue::ClassAddition(_) => todo!(),
                        crate::animations::AnimationValue::ClassRemoval(class_name) => {
                            writeln!(
                                w,
                                "    document.getElementById('{id}').classList.remove('{class_name}');"
                            )?;
                        }
                        crate::animations::AnimationValue::FieldChange { .. } => todo!(),
                    }

                    writeln!(w, "}},\n reverse: () => {{")?;

                    match &animation.value {
                        crate::animations::AnimationValue::ClassAddition(_) => todo!(),
                        crate::animations::AnimationValue::ClassRemoval(class_name) => {
                            writeln!(
                                w,
                                "    document.getElementById('{id}').classList.add('{class_name}');"
                            )?;
                        }
                        crate::animations::AnimationValue::FieldChange { .. } => todo!(),
                    }

                    writeln!(w, "}}\n}});")?;
                }
            }
        }
        Ok(())
    }

    pub fn apply_to_styling<S>(&self, styling: &mut crate::ElementStyling<S>) {
        let animation_init_values: Vec<_> = self
            .animations
            .iter()
            .filter_map(|a| match &a.value {
                crate::animations::AnimationValue::ClassAddition(_) => None,
                crate::animations::AnimationValue::ClassRemoval(_) => None,
                crate::animations::AnimationValue::FieldChange {
                    field_name,
                    previous_value,
                    ..
                } => Some((field_name.clone(), previous_value.clone())),
            })
            .collect();
        assert!(animation_init_values.is_empty());
    }
}
