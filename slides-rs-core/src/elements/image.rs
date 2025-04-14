use std::{fmt::Display, path::PathBuf};

use crate::{
    ElementStyling, ImageStyling, Result, StylingReference, ToCss, output::PresentationEmitter,
};

use super::{ElementId, WebRenderable, WebRenderableContext};

#[derive(Debug, Clone)]
pub struct Image {
    namespace: String,
    name: String,
    id: ElementId,
    parent: Option<ElementId>,
    source: ImageSource,
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
            namespace: String::new(),
            name: String::new(),
            id: ElementId::generate(),
            parent: None,
            source,
            styling: ImageStyling::new(),
            stylings: Vec::new(),
        }
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

    pub fn add_styling(&mut self, reference: StylingReference) {
        self.stylings.push(reference);
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.element_styling().class_name(), self.id)
        } else {
            self.name.clone()
        }
    }
}

impl WebRenderable for Image {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        self.styling
            .to_css_rule(ctx.layout, &format!("#{id}"), emitter.raw_css())?;
        self.source.add_files(emitter)?;
        writeln!(
            emitter.raw_html(),
            "<img id=\"{id}\" class=\"image\" src=\"{}\"/>",
            self.source
        )?;
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
        self.element_styling_mut().base_mut()
    }

    fn element_styling(&self) -> &crate::BaseElementStyling {
        self.styling.base()
    }
}
