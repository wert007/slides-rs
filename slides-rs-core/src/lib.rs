use std::{io::Write, marker::PhantomData, path::PathBuf};

pub type Result<T> = std::result::Result<T, error::SlidesError>;

const BASE_STYLE: &str = include_str!("../assets/style.css");
const NAVIGATION_JS: &str = include_str!("../assets/navigation.js");

mod error;
mod layout;
pub use layout::*;
mod styling;
use output::PresentationEmitter;
pub use styling::*;
mod elements;
pub use elements::*;
mod output;

pub struct Index<T> {
    marker: PhantomData<T>,
    value: usize,
}
impl<T> Index<T> {
    unsafe fn new(index: usize) -> Index<T> {
        Self {
            marker: PhantomData::default(),
            value: index,
        }
    }
}

pub struct Presentation {
    slides: Vec<Slide>,
    stylings: Vec<DynamicElementStyling>,
}

impl Presentation {
    pub fn new() -> Self {
        Self {
            slides: Vec::new(),
            stylings: Vec::new(),
        }
    }

    pub fn add_slide(&mut self, slide: Slide) -> Index<Slide> {
        let index = self.slides.len();
        self.slides.push(slide);
        unsafe { Index::new(index) }
    }

    pub fn output_to_directory(self, directory: impl Into<PathBuf>) -> Result<()> {
        let directory: PathBuf = directory.into();
        let mut emitter = PresentationEmitter::new(directory)?;
        writeln!(
            emitter.raw_html(),
            r#"<html>
            <head>
            <link href="style.css" rel="stylesheet"/>
            <script src="navigation.js"></script>
            </head>
            <body onload="init()" onkeydown="keydown(event)">"#
        )?;
        for (index, mut slide) in self.slides.into_iter().enumerate() {
            slide.set_fallback_id(format!("slide-{index}"));
            slide.output_to_html(&mut emitter)?
        }

        for styling in self.stylings {
            writeln!(emitter.raw_css(), ".{} {{", styling.name())?;
            writeln!(
                emitter.raw_css(),
                "{}",
                styling.to_css_style().expect("should not be empty!")
            )?;
            writeln!(emitter.raw_css(), "}}")?;
        }
        emitter.copy_referenced_files()?;
        writeln!(emitter.raw_html(), "</body></html>")?;
        Ok(())
    }

    pub fn add_styling<S: ToCss + 'static>(
        &mut self,
        styling: ElementStyling<S>,
        name: impl Into<String>,
    ) -> StylingReference {
        let name = name.into();
        self.stylings.push(styling.to_dynamic(name.clone()));
        unsafe { StylingReference::from_raw(name) }
    }
}

pub struct Slide {
    id: Option<String>,
    elements: Vec<Element>,
    styling: SlideStyling,
}

impl Slide {
    pub fn new() -> Self {
        Self {
            id: None,
            elements: Vec::new(),
            styling: SlideStyling::default(),
        }
    }

    pub fn with_styling(mut self, styling: SlideStyling) -> Self {
        self.styling = styling;
        self
    }

    pub fn add_label(mut self, label: Label) -> Slide {
        self.elements.push(Element::Label(label));
        self
    }

    pub fn add_image(mut self, image: Image) -> Slide {
        self.elements.push(Element::Image(image));
        self
    }

    fn output_to_html<W: Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        let id = self.id.expect("id should have been set here!");
        let style = self.styling.to_css_style();
        if let Some(style) = style {
            writeln!(emitter.raw_css(), "#{id} {{ {style} }}")?;
        }
        writeln!(emitter.raw_html(), "<section id=\"{id}\" class=\"slide\">")?;
        for (index, mut element) in self.elements.into_iter().enumerate() {
            element.set_fallback_id(format!("element-{index}"));
            element.set_parent_id(id.clone());
            element.output_to_html(emitter)?;
        }
        writeln!(emitter.raw_html(), "</section>")?;
        Ok(())
    }

    fn set_fallback_id(&mut self, fallback: String) {
        self.id.get_or_insert(fallback);
    }
}
