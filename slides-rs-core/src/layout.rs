use std::ops::Add;

#[derive(Debug)]
pub struct Positioning {
    vertical_alignment: VerticalAlignment,
    horizontal_alignment: HorizontalAlignment,
    margin: Thickness,
    padding: Thickness,
}

impl Positioning {
    pub fn new() -> Self {
        Self {
            vertical_alignment: VerticalAlignment::Top,
            horizontal_alignment: HorizontalAlignment::Left,
            margin: Thickness::UNSPECIFIED,
            padding: Thickness::UNSPECIFIED,
        }
    }

    pub fn with_alignment_center(mut self) -> Self {
        self.vertical_alignment = VerticalAlignment::Center;
        self.horizontal_alignment = HorizontalAlignment::Center;
        self
    }

    pub fn top(&self) -> StyleUnit {
        self.margin.top + self.padding.top
    }

    pub fn bottom(&self) -> StyleUnit {
        self.margin.bottom + self.padding.bottom
    }

    pub fn left(&self) -> StyleUnit {
        self.margin.left + self.padding.left
    }

    pub fn right(&self) -> StyleUnit {
        self.margin.right + self.padding.right
    }

    pub(crate) fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        let mut translate = (0.0, 0.0);

        match self.vertical_alignment {
            VerticalAlignment::Top => {
                writeln!(result, "top: {};", self.top()).expect("infallible");
            }
            VerticalAlignment::Center => {
                writeln!(result, "top: 50%;").expect("infallible");
                translate.1 = -50.0;
            }
            VerticalAlignment::Bottom => {
                writeln!(result, "bottom: {};", self.bottom()).expect("infallible");
            }
            VerticalAlignment::Stretch => {
                writeln!(
                    result,
                    "top: {};\nbottom: {};\nheight: 100%;",
                    self.top(),
                    self.bottom()
                )
                .expect("infallible");
            }
        }

        match self.horizontal_alignment {
            HorizontalAlignment::Left => {
                writeln!(result, "left: {};", self.left()).expect("infallible");
            }
            HorizontalAlignment::Center => {
                writeln!(result, "left: 50%;").expect("infallible");
                translate.0 = -50.0;
            }
            HorizontalAlignment::Right => {
                writeln!(result, "right: {};", self.right()).expect("infallible");
            }
            HorizontalAlignment::Stretch => {
                writeln!(
                    result,
                    "left: {};\nbottom: {};\nwidth: 100%;",
                    self.left(),
                    self.right()
                )
                .expect("infallible");
            }
        }

        if translate != (0.0, 0.0) {
            writeln!(
                result,
                "transform: translate({}%, {}%);",
                translate.0, translate.1
            )
            .expect("infallible");
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn with_alignment_stretch(mut self) -> Positioning {
        self.vertical_alignment = VerticalAlignment::Stretch;
        self.horizontal_alignment = HorizontalAlignment::Stretch;
        self
    }
}

#[derive(Debug)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
    Stretch,
}

#[derive(Debug)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
    Stretch,
}

#[derive(Debug)]
pub struct Thickness {
    left: StyleUnit,
    top: StyleUnit,
    right: StyleUnit,
    bottom: StyleUnit,
}

impl Thickness {
    const UNSPECIFIED: Thickness = Thickness {
        left: StyleUnit::Unspecified,
        top: StyleUnit::Unspecified,
        right: StyleUnit::Unspecified,
        bottom: StyleUnit::Unspecified,
    };
}

#[derive(Debug, strum::Display, Clone, Copy)]
pub enum StyleUnit {
    #[strum(to_string = "unset")]
    Unspecified,
    #[strum(to_string = "{0}px")]
    Pixel(f64),
}

impl StyleUnit {
    fn add_pixel(&self, px: f64) -> StyleUnit {
        match self {
            StyleUnit::Unspecified => StyleUnit::Pixel(px),
            StyleUnit::Pixel(spx) => StyleUnit::Pixel(spx + px),
        }
    }
}

impl Add for StyleUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            StyleUnit::Unspecified => self,
            StyleUnit::Pixel(px) => self.add_pixel(px),
        }
    }
}
