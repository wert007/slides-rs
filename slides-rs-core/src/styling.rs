use std::fmt::Display;

#[derive(Debug, Default)]
pub struct SlideStyling {
    background: Background,
}
impl SlideStyling {
    pub fn with_background(mut self, background: Background) -> SlideStyling {
        self.background = background;
        self
    }

    pub(crate) fn to_css_style(&self) -> Option<String> {
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
}

#[derive(Debug, Default)]
pub struct LabelStyling {
    background: Background,
    text_color: Option<Color>,
}
impl LabelStyling {
    pub fn with_background(mut self, background: Background) -> LabelStyling {
        self.background = background;
        self
    }

    pub fn with_text_color(mut self, text_color: Color) -> LabelStyling {
        self.text_color = Some(text_color);
        self
    }

    pub(crate) fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background).expect("infallible");
        }
        if let Some(text_color) = self.text_color {
            writeln!(result, "color: {text_color};").expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
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
    background: Background,
    object_fit: ObjectFit,
}

impl ImageStyling {
    pub fn with_background(mut self, background: Background) -> ImageStyling {
        self.background = background;
        self
    }

    pub(crate) fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background).expect("infallible");
        }
        if self.object_fit != ObjectFit::Unspecified {
            writeln!(result, "object-fit: {};", self.object_fit).expect("infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn with_object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.object_fit = object_fit;
        self
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
