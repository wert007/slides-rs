use std::fmt::Display;

use crate::{
    ElementStyling, LabelStyling, Result, StylingReference, ToCss, output::PresentationEmitter,
};

use super::WebRenderable;

#[derive(Debug, Clone)]
pub struct FormattedText {
    text: String,
}
impl FormattedText {
    fn render_to_html<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        writeln!(w, "{}", markdown::to_html(&self.text))?;
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
    parent_id: String,
    id: String,
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

    fn output_to_html<W: std::io::Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        let id = format!("{}-{}", self.parent_id, self.id);
        let style_styling = self.styling.to_css_style();
        writeln!(emitter.raw_css(), "#{id} {{")?;
        writeln!(emitter.raw_css(), "{style_styling}}}")?;
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

    fn set_fallback_id(&mut self, id: String) {
        if self.id.is_empty() {
            self.id = id;
        }
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn set_parent_id(&mut self, id: String) {
        self.parent_id = id;
    }

    fn element_styling_mut(&mut self) -> &mut crate::BaseElementStyling {
        self.styling.base_mut()
    }
}

impl Label {
    pub fn new(text: impl Into<FormattedText>) -> Self {
        Self {
            parent_id: String::new(),
            id: String::new(),
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

    pub fn as_element_mut(&mut self) -> super::ElementRefMut {
        super::ElementRefMut::Label(self)
    }

    pub fn add_styling(&mut self, reference: StylingReference) {
        self.stylings.push(reference);
    }
}
