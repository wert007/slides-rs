use std::{borrow::Cow, fmt::Display, path::PathBuf};

use enum_dispatch::enum_dispatch;

use crate::{ImageStyling, LabelStyling, Positioning, Result, output::PresentationEmitter};

#[enum_dispatch]
pub trait WebRenderable {
    fn output_to_html<W: std::io::Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()>;
    fn set_fallback_id(&mut self, id: String);

    fn set_parent_id(&mut self, id: String);
}

#[enum_dispatch(WebRenderable)]
pub enum Element {
    Image,
    Label,
}

pub struct Image {
    id: Option<String>,
    source: ImageSource,
    positioning: Positioning,
    styling: ImageStyling,
}

pub enum ImageSource {
    Path(PathBuf),
}

impl Display for ImageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageSource::Path(path_buf) => write!(f, "{}", path_buf.display()),
        }
    }
}

impl Image {
    pub fn new(source: ImageSource) -> Self {
        Self {
            id: None,
            source,
            positioning: Positioning::new(),
            styling: ImageStyling::default(),
        }
    }
}

impl WebRenderable for Image {
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
            "<img id=\"{id}\" class=\"image\" src=\"{}\"/>",
            self.source
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

pub struct Label {
    id: Option<String>,
    text: Cow<'static, str>,
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
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
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
