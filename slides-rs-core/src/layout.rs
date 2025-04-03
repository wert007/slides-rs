use std::{fmt::Display, num::ParseFloatError, ops::Add, str::FromStr};

use crate::SlidesEnum;

#[derive(Debug, Clone, Copy)]
pub struct Positioning {
    z_value: Option<usize>,
    vertical_alignment: VerticalAlignment,
    horizontal_alignment: HorizontalAlignment,
    margin: Thickness,
    padding: Thickness,
}

impl Positioning {
    pub fn new() -> Self {
        Self {
            z_value: None,
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

    pub fn with_padding(mut self, padding: Thickness) -> Self {
        self.padding = padding;
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

    pub(crate) fn to_css_style(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut translate = (0.0, 0.0);

        match self.vertical_alignment {
            VerticalAlignment::Top => {
                writeln!(result, "top: {};", self.margin.top.or_zero()).expect("infallible");
            }
            VerticalAlignment::Center => {
                writeln!(result, "top: 50%;").expect("infallible");
                translate.1 = -50.0;
            }
            VerticalAlignment::Bottom => {
                writeln!(result, "bottom: {};", self.margin.bottom.or_zero()).expect("infallible");
            }
            VerticalAlignment::Stretch => {
                writeln!(
                    result,
                    "top: {};\nbottom: {};\nheight: 100%;",
                    self.margin.top.or_zero(),
                    self.margin.bottom.or_zero()
                )
                .expect("infallible");
            }
        }

        match self.horizontal_alignment {
            HorizontalAlignment::Left => {
                writeln!(result, "left: {};", self.margin.left.or_zero()).expect("infallible");
            }
            HorizontalAlignment::Center => {
                writeln!(result, "left: 50%;").expect("infallible");
                translate.0 = -50.0;
            }
            HorizontalAlignment::Right => {
                writeln!(result, "right: {};", self.margin.right.or_zero()).expect("infallible");
            }
            HorizontalAlignment::Stretch => {
                writeln!(
                    result,
                    "left: {};\nbottom: {};\nwidth: 100%;",
                    self.margin.left.or_zero(),
                    self.margin.right.or_zero()
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

        if self.padding != Thickness::UNSPECIFIED {
            writeln!(result, "padding: {};", self.padding).expect("infallible");
        }

        if let Some(z_value) = self.z_value {
            writeln!(result, "z-index: {z_value};").expect("infallible");
        }

        result
    }

    pub fn with_alignment_stretch(mut self) -> Positioning {
        self.vertical_alignment = VerticalAlignment::Stretch;
        self.horizontal_alignment = HorizontalAlignment::Stretch;
        self
    }

    pub fn set_vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) {
        self.vertical_alignment = vertical_alignment;
    }

    pub fn set_horizontal_alignment(&mut self, horizontal_alignment: HorizontalAlignment) {
        self.horizontal_alignment = horizontal_alignment;
    }
}

#[derive(Debug, Clone, Copy, strum::VariantNames, strum::EnumString)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
    Stretch,
}

impl SlidesEnum for VerticalAlignment {}

#[derive(Debug, Clone, Copy, strum::VariantNames, strum::EnumString)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
    Stretch,
}

impl SlidesEnum for HorizontalAlignment {}

#[derive(Debug, PartialEq, Clone, Copy)]
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

    pub fn all(value: StyleUnit) -> Thickness {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }
}

impl Display for Thickness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.left, self.top, self.right, self.bottom
        )
    }
}

#[derive(Debug, strum::Display, Clone, Copy, PartialEq)]
pub enum StyleUnit {
    #[strum(to_string = "unset")]
    Unspecified,
    #[strum(to_string = "{0}px")]
    Pixel(f64),
    #[strum(to_string = "{0}pt")]
    Point(f64),
    #[strum(to_string = "{0}%")]
    Percent(f64),
}

impl StyleUnit {
    fn add_pixel(&self, px: f64) -> StyleUnit {
        match self {
            StyleUnit::Unspecified => StyleUnit::Pixel(px),
            StyleUnit::Pixel(spx) => StyleUnit::Pixel(spx + px),
            StyleUnit::Point(pt) => todo!(),
            StyleUnit::Percent(percent) => todo!(),
        }
    }

    fn or_zero(&self) -> StyleUnit {
        match self {
            Self::Unspecified => StyleUnit::Pixel(0.0),
            normal => *normal,
        }
    }
}

impl Add for StyleUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            StyleUnit::Unspecified => self,
            StyleUnit::Pixel(px) => self.add_pixel(px),
            StyleUnit::Point(pt) => todo!(),
            StyleUnit::Percent(percent) => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum StyleUnitParseError {
    ParseFloatError(ParseFloatError),
    UnknownUnits,
}

impl FromStr for StyleUnit {
    type Err = StyleUnitParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_index = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .count();
        let (number, unit) = s.split_at(split_index);
        let number = f64::from_str(number).map_err(|e| StyleUnitParseError::ParseFloatError(e))?;
        Ok(match unit {
            "%" => StyleUnit::Percent(number),
            "px" => StyleUnit::Pixel(number),
            "pt" => StyleUnit::Point(number),
            _ => return Err(StyleUnitParseError::UnknownUnits),
        })
    }
}
