use std::fmt::Display;

use crate::{
    ElementStyling, LabelStyling, Result, StylingReference, ToCss, output::PresentationEmitter,
};

use super::{ElementId, WebRenderable, WebRenderableContext};

mod markdown;

#[derive(Debug, Clone)]
pub struct FormattedText {
    text: String,
}

impl FormattedText {
    fn render_to_html<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        markdown::render_markdown(
            w,
            markdown::to_mdast(&self.text, &markdown::SLIDE_MARKDOWN_PARSE_OPTIONS).unwrap(),
        )?;
        Ok(())
    }
}

impl<T: Into<String>> From<T> for FormattedText {
    fn from(value: T) -> Self {
        Self { text: value.into() }
    }
}

impl Display for FormattedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    namespace: String,
    name: String,
    id: ElementId,
    parent: Option<ElementId>,
    text: FormattedText,
    styling: ElementStyling<LabelStyling>,
    stylings: Vec<StylingReference>,
}

impl WebRenderable for Label {
    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        self.styling.collect_google_font_references(fonts)
    }

    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        self.styling
            .to_css_rule(ctx.layout, &format!("#{id}"), emitter.raw_css())?;
        // TODO: Maybe in future all text is gonna be inside of svg to allow
        // seemless scaling.
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"label{}\">",
            self.stylings
                .into_iter()
                .map(|s| format!(" {s}"))
                .collect::<String>()
        )?;
        self.text.render_to_html(emitter.raw_html())?;
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

    fn set_name(&mut self, id: String) {
        self.name = id;
    }

    fn set_namespace(&mut self, id: String) {
        self.namespace = id;
    }

    fn element_styling_mut(&mut self) -> &mut crate::BaseElementStyling {
        self.styling.base_mut()
    }

    fn element_styling(&self) -> &crate::BaseElementStyling {
        self.styling.base()
    }
}

impl Label {
    pub fn new(text: impl Into<FormattedText>) -> Self {
        Self {
            namespace: String::new(),
            name: String::new(),
            id: ElementId::generate(),
            parent: None,
            text: text.into(),
            styling: LabelStyling::new(),
            stylings: Vec::new(),
        }
    }

    pub fn with_element_styling(mut self, styling: ElementStyling<LabelStyling>) -> Self {
        self.styling = styling;
        self
    }

    pub fn with_styling(mut self, styling: StylingReference) -> Label {
        self.stylings.push(styling);
        self
    }

    pub fn element_styling_mut(&mut self) -> &mut ElementStyling<LabelStyling> {
        &mut self.styling
    }

    pub fn add_styling(&mut self, reference: StylingReference) {
        self.stylings.push(reference);
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.styling.class_name(), self.id)
        } else {
            self.name.clone()
        }
    }
}
