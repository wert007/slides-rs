use std::io::Write;

use crate::{
    ElementStyling, Result, StylingReference, ToCss, animations::Animations,
    output::PresentationEmitter,
};

use super::{Element, ElementId, WebRenderable, WebRenderableContext};

#[derive(Debug, Clone)]
pub struct CustomElement {
    namespace: String,
    name: String,
    id: ElementId,
    parent: Option<ElementId>,
    styling: ElementStyling<()>,
    stylings: Vec<StylingReference>,
    type_name: String,
    children: Vec<Element>,
    pub animations: Animations,
}
impl CustomElement {
    pub fn new(type_name: impl Into<String>, children: Vec<Element>) -> Self {
        Self {
            namespace: String::new(),
            name: String::new(),
            id: ElementId::generate(),
            parent: None,
            type_name: type_name.into(),
            children,
            styling: ElementStyling::new_base(),
            stylings: Vec::new(),
            animations: Animations::new(),
        }
    }
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn element_styling_mut(&mut self) -> &mut ElementStyling<()> {
        &mut self.styling
    }

    pub fn with_element_styling(mut self, styling: ElementStyling<()>) -> Self {
        self.styling = styling;
        self
    }

    pub fn add_styling(&mut self, reference: StylingReference) {
        self.stylings.push(reference);
    }

    pub fn with_elements(mut self, children: Vec<Element>) -> CustomElement {
        self.children = children;
        self
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.styling.class_name(), self.id)
        } else {
            self.name.clone()
        }
    }

    pub fn element_by_name(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|e| e.name() == name)
    }
}

impl WebRenderable for CustomElement {
    fn output_to_html<W: Write>(
        mut self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        let classes_animations = self.animations.get_initial_classes();
        let classes = self
            .stylings
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        self.animations
            .emit_to_javascript(emitter.raw_js(), ctx.clone(), &id)?;
        self.animations.apply_to_styling(&mut self.styling);
        self.styling
            .to_css_rule(ctx.layout.clone(), &format!("#{id}"), emitter.raw_css())?;
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"custom-element {} {classes} {classes_animations}\">",
            self.type_name,
        )?;
        for element in self.children {
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

    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, id: String) {
        self.name = id;
        self.children
            .iter_mut()
            .for_each(|c| c.set_namespace(format!("{}-{}", self.namespace, self.name)));
    }

    fn namespace(&self) -> String {
        self.namespace.clone()
    }

    fn set_namespace(&mut self, id: String) {
        self.namespace = id;
        self.children
            .iter_mut()
            .for_each(|c| c.set_namespace(format!("{}-{}", self.namespace, self.name)));
    }

    fn element_styling_mut(&mut self) -> &mut crate::BaseElementStyling {
        self.element_styling_mut().base_mut()
    }

    fn element_styling(&self) -> &crate::BaseElementStyling {
        self.styling.base()
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> crate::Result<()> {
        for child in &self.children {
            child.collect_google_font_references(fonts)?;
        }
        Ok(())
    }
}
