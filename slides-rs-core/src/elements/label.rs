use std::fmt::Display;

use crate::{
    ElementStyling, LabelStyling, Positioning, Result, StylingReference, ToCss,
    output::PresentationEmitter,
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
    id: Option<String>,
    text: FormattedText,
    positioning: Positioning,
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
        let id = self.id.expect("id should have been set here!");
        let style_positioning = self.positioning.to_css_style();
        let style_styling = self.styling.to_css_style();
        let style = match (style_positioning, style_styling) {
            (None, None) => None,
            (None, b) => b,
            (a, None) => a,
            (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
        };
        if let Some(style) = style {
            writeln!(emitter.raw_css(), "#{id} {{\n{style}\n}}")?;
        }
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
        self.id.get_or_insert(id);
    }

    fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    fn set_parent_id(&mut self, id: String) {
        self.id = Some(format!(
            "{id}-{}",
            self.id.as_ref().expect("call set_fallback_id before")
        ));
    }
}

impl Label {
    pub fn new(text: impl Into<FormattedText>) -> Self {
        Self {
            id: None,
            text: text.into(),
            positioning: Positioning::new(),
            styling: LabelStyling::new(),
            stylings: Vec::new(),
        }
    }

    pub fn with_positioning(mut self, positioning: Positioning) -> Self {
        self.positioning = positioning;
        self
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
}
