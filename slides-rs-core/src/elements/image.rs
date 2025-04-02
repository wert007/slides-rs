use std::{fmt::Display, path::PathBuf};

use crate::{
    ElementStyling, ImageStyling, Positioning, Result, StylingReference, ToCss,
    output::PresentationEmitter,
};

use super::WebRenderable;

#[derive(Debug, Clone)]
pub struct Image {
    z_index: usize,
    parent_id: String,
    id: String,
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
            z_index: 0,
            parent_id: String::new(),
            id: String::new(),
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

    pub fn element_styling_mut(&mut self) -> &mut ElementStyling<ImageStyling> {
        &mut self.styling
    }

    pub fn positioning_mut(&mut self) -> &mut Positioning {
        &mut self.positioning
    }

    pub fn as_element_mut(&mut self) -> super::ElementRefMut<'_> {
        super::ElementRefMut::Image(self)
    }
}

impl WebRenderable for Image {
    fn output_to_html<W: std::io::Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        let id = format!("{}-{}", self.parent_id, self.id);
        let style_positioning = self.positioning.to_css_style();
        let style_styling = self.styling.to_css_style();
        writeln!(
            emitter.raw_css(),
            "#{id} {{\nz-index: {};\n{style_positioning}\n{style_styling}\n}}",
            self.z_index
        )?;
        self.source.add_files(emitter)?;
        writeln!(
            emitter.raw_html(),
            "<img id=\"{id}\" class=\"image\" src=\"{}\"/>",
            self.source
        )?;
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

    fn set_z_index(&mut self, z_index: usize) {
        self.z_index = z_index;
    }
}
