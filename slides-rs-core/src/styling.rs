use crate::Result;
use std::{fmt::Display, ops::Deref};

pub trait ToCss {
    fn to_css_style(&self) -> Option<String>;

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct StylingReference {
    name: String,
}

impl StylingReference {
    pub unsafe fn from_raw(name: String) -> StylingReference {
        Self { name }
    }
}

impl Display for StylingReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct DynamicElementStyling {
    name: String,
    base: BaseElementStyling,
    specific: Box<dyn ToCss>,
}

impl DynamicElementStyling {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl ToCss for DynamicElementStyling {
    fn to_css_style(&self) -> Option<String> {
        let base = self.base.to_css_style();
        let specific = self.specific.to_css_style();
        match (base, specific) {
            (None, None) => None,
            (None, it) => it,
            (it, None) => it,
            (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
        }
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        self.specific.collect_google_font_references(fonts)
    }
}

#[derive(Debug, Default)]
pub struct BaseElementStyling {
    background: Background,
}

impl ToCss for BaseElementStyling {
    fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background).expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ElementStyling<S> {
    base: BaseElementStyling,
    specific: S,
}

impl<S> Deref for ElementStyling<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.specific
    }
}

impl<S: ToCss + 'static> ElementStyling<S> {
    pub fn with_background(mut self, background: Background) -> Self {
        self.base.background = background;
        self
    }

    fn new(specific: S) -> Self {
        Self {
            base: BaseElementStyling::default(),
            specific,
        }
    }

    pub fn to_dynamic(self, name: String) -> DynamicElementStyling {
        DynamicElementStyling {
            name,
            base: self.base,
            specific: Box::new(self.specific),
        }
    }
}

impl<S: ToCss> ToCss for ElementStyling<S> {
    fn to_css_style(&self) -> Option<String> {
        let base = self.base.to_css_style();
        let specific = self.specific.to_css_style();
        match (base, specific) {
            (None, None) => None,
            (None, it) => it,
            (it, None) => it,
            (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
        }
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        self.specific.collect_google_font_references(fonts)
    }
}

#[derive(Debug, Default)]
pub struct SlideStyling {
    background: Background,
}
impl SlideStyling {
    pub fn with_background(mut self, background: Background) -> SlideStyling {
        self.background = background;
        self
    }
}

impl ToCss for SlideStyling {
    fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background).expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, strum::Display)]
pub enum Font {
    #[strum(to_string = "unset")]
    Unspecified,
    #[strum(to_string = "\"{0}\"")]
    GoogleFont(String),
    #[strum(to_string = "\"{0}\"")]
    System(String),
}

impl Font {
    pub fn gfont(name: impl Into<String>) -> Self {
        Self::GoogleFont(name.into())
    }

    pub fn system(name: impl Into<String>) -> Self {
        Self::System(name.into())
    }
}

#[derive(Debug)]
pub struct LabelStyling {
    text_color: Option<Color>,
    font: Font,
}

impl LabelStyling {
    pub fn new() -> ElementStyling<LabelStyling> {
        ElementStyling::new(Self {
            text_color: None,
            font: Font::Unspecified,
        })
    }
}

impl ElementStyling<LabelStyling> {
    pub fn with_text_color(mut self, text_color: Color) -> Self {
        self.specific.text_color = Some(text_color);
        self
    }

    pub fn with_font(mut self, font: Font) -> Self {
        self.specific.font = font;
        self
    }

    pub fn header_data(&self) -> Option<String> {
        match &self.font {
            Font::Unspecified => None,
            Font::GoogleFont(name) => Some(format!(
                r#"<link href="https://fonts.googleapis.com/css2?family={name}" rel="stylesheet">"#
            )),
            Font::System(_) => None,
        }
    }
}

impl ToCss for LabelStyling {
    fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if let Some(text_color) = self.text_color {
            writeln!(result, "color: {text_color};").expect("infallible");
        }
        if self.font != Font::Unspecified {
            writeln!(result, "font-family: {};", self.font).expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        match &self.font {
            Font::GoogleFont(name) => {
                fonts.insert(name.clone());
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum ObjectFit {
    #[default]
    #[strum(to_string = "unset")]
    Unspecified,
    Contain,
    None,
    Cover,
    Fill,
    ScaleDown,
}

#[derive(Debug, Default)]
pub struct ImageStyling {
    object_fit: ObjectFit,
}

impl ImageStyling {
    pub fn new() -> ElementStyling<ImageStyling> {
        ElementStyling::new(ImageStyling::default())
    }
}

impl ElementStyling<ImageStyling> {
    pub fn with_object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.specific.object_fit = object_fit;
        self
    }
}

impl ToCss for ImageStyling {
    fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.object_fit != ObjectFit::Unspecified {
            writeln!(result, "object-fit: {};", self.object_fit).expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Background {
    #[default]
    Unspecified,
    Color(Color),
}

impl Display for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Background::Unspecified => write!(f, "unset"),
            Background::Color(color) => write!(f, "{color}"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
}

impl Color {
    pub const WHITE: Color = Self::from_rgb(0xff, 0xff, 0xff);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            alpha: 0xff,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgb({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            self.alpha as f64 / 255.0
        )
    }
}
