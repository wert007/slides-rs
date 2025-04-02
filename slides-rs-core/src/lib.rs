use std::{collections::HashSet, io::Write, marker::PhantomData, path::PathBuf};

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

#[allow(dead_code)]
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

            <!-- For Google font! -->
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>"#
        )?;

        let mut google_font_references = HashSet::new();
        for slide in &self.slides {
            slide.collect_google_font_references(&mut google_font_references)?;
        }

        for styling in &self.stylings {
            styling.collect_google_font_references(&mut google_font_references)?;
        }

        for google_font in google_font_references {
            writeln!(
                emitter.raw_html(),
                r#"<link href="https://fonts.googleapis.com/css2?family={google_font}" rel="stylesheet">"#
            )?;
        }

        writeln!(
            emitter.raw_html(),
            r#"</head>
            <body onload="init()" onkeydown="keydown(event)">"#
        )?;
        for (index, mut slide) in self.slides.into_iter().enumerate() {
            slide.set_fallback_id(format!("slide-{index}"));
            slide.output_to_html(&mut emitter)?
        }

        for styling in self.stylings {
            let css = styling.to_css_style();
            if css.is_empty() {
                continue;
            }
            writeln!(emitter.raw_css(), ".{} {{", styling.name())?;
            writeln!(emitter.raw_css(), "{css}")?;
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

    pub fn set_default_style<S: ToCss + 'static>(&mut self, styling: ElementStyling<S>) {
        let name = styling.class_name();
        self.stylings.push(styling.to_dynamic(name));
    }
}

pub struct Slide {
    id: Option<String>,
    elements: Vec<Element>,
    styling: ElementStyling<SlideStyling>,
    current_z_index: usize,
}

impl Slide {
    pub fn new() -> Self {
        Self {
            id: None,
            elements: Vec::new(),
            styling: SlideStyling::new(),
            current_z_index: 0,
        }
    }

    pub fn with_styling(mut self, styling: ElementStyling<SlideStyling>) -> Self {
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
        if !style.is_empty() {
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

    fn collect_google_font_references(&self, fonts: &mut HashSet<String>) -> Result<()> {
        for element in &self.elements {
            element.collect_google_font_references(fonts)?;
        }
        Ok(())
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Slide {
        self.id = Some(name.into());
        self
    }

    pub fn styling_mut(&mut self) -> &mut ElementStyling<SlideStyling> {
        &mut self.styling
    }

    pub fn add_custom_element(mut self, custom_element: CustomElement) -> Slide {
        self.elements.push(custom_element.into());
        self
    }

    pub fn next_z_index(&mut self) -> usize {
        let result = self.current_z_index;
        self.current_z_index += 1;
        result
    }
}
