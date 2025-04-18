use crate::{
    BaseElementStyling, ElementStyling, FlexStyling, PresentationEmitter, Result, StylingReference,
    ToCss,
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
}

impl Flex {
    pub fn new(children: Vec<Element>) -> Self {
        Self {
            namespace: String::new(),
            name: String::new(),
            id: ElementId::generate(),
            parent: None,
            children,
            styling: FlexStyling::new(),
            stylings: Vec::new(),
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
        ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        self.styling
            .to_css_rule(ctx.layout, &format!("#{id}"), emitter.raw_css())?;
        writeln!(emitter.raw_html(), "<div id=\"{id}\" class=\"flex\">")?;
        for mut element in self.children {
            element.set_namespace(id.clone());
            // element.set_fallback_id(index.to_string());

            element.output_to_html(emitter, ctx)?;
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
