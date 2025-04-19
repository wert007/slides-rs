use crate::{
    BaseElementStyling, ElementStyling, FlexStyling, PresentationEmitter, Result, StylingReference,
    ToCss,
    animations::{Animation, Animations},
};

use super::{Element, ElementId, WebRenderable, WebRenderableContext};

#[derive(Debug, Clone)]
pub struct Flex {
    namespace: String,
    name: String,
    id: ElementId,
    parent: Option<ElementId>,
    children: Vec<Element>,
    styling: ElementStyling<FlexStyling>,
    stylings: Vec<StylingReference>,
    pub animations: Vec<Animation>,
}

impl Flex {
    pub fn new(mut children: Vec<Element>) -> Self {
        let id = ElementId::generate();
        children.iter_mut().for_each(|c| c.set_parent(id));
        Self {
            namespace: String::new(),
            name: String::new(),
            id,
            parent: None,
            children,
            styling: FlexStyling::new(),
            stylings: Vec::new(),
            animations: Vec::new(),
        }
    }

    pub fn styling_mut(&mut self) -> &mut ElementStyling<FlexStyling> {
        &mut self.styling
    }

    pub fn add_styling(&mut self, styling: StylingReference) {
        self.stylings.push(styling);
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.styling.class_name(), self.id)
        } else {
            self.name.clone()
        }
    }
}

impl WebRenderable for Flex {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        mut ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
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
        let classes = self
            .stylings
            .into_iter()
            .map(|s| s.to_string())
            .chain(self.animations.iter().filter_map(|a| match &a.value {
                crate::animations::AnimationValue::ClassAddition(_) => None,
                crate::animations::AnimationValue::ClassRemoval(class) => Some(class.clone()),
                crate::animations::AnimationValue::FieldChange { .. } => None,
            }))
            .collect::<Vec<_>>()
            .join(" ");
        for animation in self.animations {
            match animation.trigger {
                crate::animations::Trigger::StepReached(number) => {
                    writeln!(
                        emitter.raw_js(),
                        "stepReached.push({{\n slideId: '{}',\n step: {number},\n trigger: () => {{",
                        ctx.slide_name
                    )?;
                    match &animation.value {
                        crate::animations::AnimationValue::ClassAddition(_) => todo!(),
                        crate::animations::AnimationValue::ClassRemoval(class_name) => {
                            writeln!(
                                emitter.raw_js(),
                                "    document.getElementById('{id}').classList.remove('{class_name}');"
                            )?;
                        }
                        crate::animations::AnimationValue::FieldChange { .. } => todo!(),
                    }

                    writeln!(emitter.raw_js(), "}},\n reverse: () => {{")?;

                    match &animation.value {
                        crate::animations::AnimationValue::ClassAddition(_) => todo!(),
                        crate::animations::AnimationValue::ClassRemoval(class_name) => {
                            writeln!(
                                emitter.raw_js(),
                                "    document.getElementById('{id}').classList.add('{class_name}');"
                            )?;
                        }
                        crate::animations::AnimationValue::FieldChange { .. } => todo!(),
                    }

                    writeln!(emitter.raw_js(), "}}\n}});")?;
                }
            }
        }
        ctx.layout.animation_init_values = animation_init_values;
        self.styling
            .to_css_rule(ctx.layout.clone(), &format!("#{id}"), emitter.raw_css())?;
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"flex {classes}\">"
        )?;
        for mut element in self.children {
            element.set_namespace(id.clone());
            // element.set_fallback_id(index.to_string());

            element.output_to_html(emitter, ctx.clone())?;
        }
        writeln!(emitter.raw_html(), "</div>")?;

        Ok(())
    }

    fn set_parent(&mut self, parent: ElementId) {
        self.parent = Some(parent);
    }

    fn parent(&self) -> Option<ElementId> {
        self.parent
    }

    fn id(&self) -> ElementId {
        self.id
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_namespace(&mut self, namespace: String) {
        self.namespace = namespace;
    }

    fn element_styling(&self) -> &BaseElementStyling {
        self.styling.base()
    }

    fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        self.styling.base_mut()
    }
}
