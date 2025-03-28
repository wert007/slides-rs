use std::{fmt::Display, path::PathBuf};

use crate::{
    ElementStyling, ImageStyling, Positioning, Result, StylingReference, ToCss,
    output::PresentationEmitter,
};

use super::WebRenderable;

#[derive(Debug, Clone)]
pub struct Image {
    id: Option<String>,
    source: ImageSource,
    positioning: Positioning,
    styling: ElementStyling<ImageStyling>,
    stylings: Vec<StylingReference>,
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Path(PathBuf),
}

impl ImageSource {
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    fn add_files<W: std::io::Write>(&self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        match self {
            ImageSource::Path(path_buf) => emitter.add_file(path_buf)?,
        }
        Ok(())
    }
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
            styling: ImageStyling::new(),
            stylings: Vec::new(),
        }
    }

    pub fn with_positioning(mut self, positioning: Positioning) -> Self {
        self.positioning = positioning;
        self
    }

    pub fn with_element_styling(mut self, styling: ElementStyling<ImageStyling>) -> Self {
        self.styling = styling;
        self
    }

    pub fn with_styling(mut self, styling: StylingReference) -> Self {
        self.stylings.push(styling);
        self
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
        self.source.add_files(emitter)?;
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
