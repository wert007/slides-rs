use std::fmt::Display;

use crate::{LabelStyling, Positioning, Result, output::PresentationEmitter};

use super::WebRenderable;

#[derive(Debug)]
pub struct FormattedText {
    text: String,
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

#[derive(Debug)]
pub struct Label {
    id: Option<String>,
    text: FormattedText,
    positioning: Positioning,
    styling: LabelStyling,
}

impl WebRenderable for Label {
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
            "<div id=\"{id}\" class=\"label\">{}</div>",
            self.text
        )?;
        Ok(())
    }

    fn set_fallback_id(&mut self, id: String) {
        self.id.get_or_insert(id);
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
            styling: LabelStyling::default(),
        }
    }

    pub fn with_positioning(mut self, positioning: Positioning) -> Self {
        self.positioning = positioning;
        self
    }

    pub fn with_styling(mut self, styling: LabelStyling) -> Self {
        self.styling = styling;
        self
    }
}
