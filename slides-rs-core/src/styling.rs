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
            writeln!(result, "background: {};", self.background)
                .expect("Writing to string is infallible");
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
}
impl LabelStyling {
    pub fn with_background(mut self, background: Background) -> LabelStyling {
        self.background = background;
        self
    }

    pub(crate) fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background)
                .expect("Writing to string is infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[derive(Debug, Default)]
pub struct ImageStyling {
    background: Background,
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
            writeln!(result, "background: {};", self.background)
                .expect("Writing to string is infallible");
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
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
