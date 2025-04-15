use std::{
    fmt::Display,
    num::ParseFloatError,
    ops::{Add, Sub},
    str::FromStr,
};

use crate::SlidesEnum;

// #[derive(Debug, Clone, Copy)]
// pub struct Positioning {
//     z_value: Option<usize>,
//     vertical_alignment: VerticalAlignment,
//     horizontal_alignment: HorizontalAlignment,
//     margin: Thickness,
//     padding: Thickness,
// }

// impl Positioning {
//     pub fn new() -> Self {
//         Self {
//             z_value: None,
//             vertical_alignment: VerticalAlignment::Top,
//             horizontal_alignment: HorizontalAlignment::Left,
//             margin: Thickness::default(),
//             padding: Thickness::default(),
//         }
//     }

//     pub fn with_alignment_center(mut self) -> Self {
//         self.vertical_alignment = VerticalAlignment::Center;
//         self.horizontal_alignment = HorizontalAlignment::Center;
//         self
//     }

//     pub fn with_padding(mut self, padding: Thickness) -> Self {
//         self.padding = padding;
//         self
//     }

//     pub fn top(&self) -> StyleUnit {
//         self.margin.top + self.padding.top
//     }

//     pub fn bottom(&self) -> StyleUnit {
//         self.margin.bottom + self.padding.bottom
//     }

//     pub fn left(&self) -> StyleUnit {
//         self.margin.left + self.padding.left
//     }

//     pub fn right(&self) -> StyleUnit {
//         self.margin.right + self.padding.right
//     }

//     pub(crate) fn to_css_style(&self) -> String {
//         use std::fmt::Write;
//         let mut result = String::new();
//         let mut translate = (0.0, 0.0);

//         match self.vertical_alignment {
//             VerticalAlignment::Top => {
//                 writeln!(result, "top: {};", self.margin.top.or_zero()).expect("infallible");
//             }
//             VerticalAlignment::Center => {
//                 writeln!(result, "top: 50%;").expect("infallible");
//                 translate.1 = -50.0;
//             }
//             VerticalAlignment::Bottom => {
//                 writeln!(result, "bottom: {};", self.margin.bottom.or_zero()).expect("infallible");
//             }
//             VerticalAlignment::Stretch => {
//                 writeln!(
//                     result,
//                     "top: {};\nbottom: {};\nheight: 100%;",
//                     self.margin.top.or_zero(),
//                     self.margin.bottom.or_zero()
//                 )
//                 .expect("infallible");
//             }
//         }

//         match self.horizontal_alignment {
//             HorizontalAlignment::Left => {
//                 writeln!(result, "left: {};", self.margin.left.or_zero()).expect("infallible");
//             }
//             HorizontalAlignment::Center => {
//                 writeln!(result, "left: 50%;").expect("infallible");
//                 translate.0 = -50.0;
//             }
//             HorizontalAlignment::Right => {
//                 writeln!(result, "right: {};", self.margin.right.or_zero()).expect("infallible");
//             }
//             HorizontalAlignment::Stretch => {
//                 writeln!(
//                     result,
//                     "left: {};\nbottom: {};\nwidth: 100%;",
//                     self.margin.left.or_zero(),
//                     self.margin.right.or_zero()
//                 )
//                 .expect("infallible");
//             }
//         }

//         if translate != (0.0, 0.0) {
//             writeln!(
//                 result,
//                 "transform: translate({}%, {}%);",
//                 translate.0, translate.1
//             )
//             .expect("infallible");
//         }

//         if self.padding != Thickness::default() {
//             writeln!(result, "padding: {};", self.padding).expect("infallible");
//         }

//         if let Some(z_value) = self.z_value {
//             writeln!(result, "z-index: {z_value};").expect("infallible");
//         }

//         result
//     }

//     pub fn with_alignment_stretch(mut self) -> Positioning {
//         self.vertical_alignment = VerticalAlignment::Stretch;
//         self.horizontal_alignment = HorizontalAlignment::Stretch;
//         self
//     }

//     pub fn set_vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) {
//         self.vertical_alignment = vertical_alignment;
//     }

//     pub fn set_horizontal_alignment(&mut self, horizontal_alignment: HorizontalAlignment) {
//         self.horizontal_alignment = horizontal_alignment;
//     }
// }

#[derive(Debug, Clone, Copy, strum::VariantNames, strum::EnumString, Default)]
pub enum VerticalAlignment {
    #[default]
    Unset,
    Top,
    Center,
    Bottom,
    Stretch,
}

impl SlidesEnum for VerticalAlignment {}

#[derive(Debug, Clone, Copy, strum::VariantNames, strum::EnumString, Default)]
pub enum HorizontalAlignment {
    #[default]
    Unset,
    Left,
    Center,
    Right,
    Stretch,
}

impl SlidesEnum for HorizontalAlignment {}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Thickness {
    pub left: StyleUnit,
    pub top: StyleUnit,
    pub right: StyleUnit,
    pub bottom: StyleUnit,
}

impl Thickness {
    pub fn all(value: impl Into<StyleUnit>) -> Thickness {
        let value = value.into();
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
            self.top, self.right, self.bottom, self.left,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CalcData {
    pixel: f64,
    percent: f64,
    point: f64,
    slide_width: f64,
    slide_height: f64,
}
impl CalcData {
    fn add_pixel(mut self, px: f64) -> Self {
        self.pixel += px;
        self
    }

    fn add_percent(mut self, percent: f64) -> CalcData {
        self.percent += percent;
        self
    }

    fn from_units(a: StyleUnit, b: StyleUnit) -> CalcData {
        CalcData::from_unit(a).apply(b)
    }

    fn from_unit(a: StyleUnit) -> Self {
        match a {
            StyleUnit::Unspecified => Self::default(),
            StyleUnit::Pixel(pixel) => Self {
                pixel,
                ..Self::default()
            },
            StyleUnit::Point(point) => Self {
                point,
                ..Self::default()
            },
            StyleUnit::Percent(percent) => Self {
                percent,
                ..Self::default()
            },
            StyleUnit::SlideWidthRatio(slide_width) => Self {
                slide_width,
                ..Self::default()
            },
            StyleUnit::SlideHeightRatio(slide_height) => Self {
                slide_height,
                ..Self::default()
            },
            StyleUnit::Calc(calc_data) => calc_data,
        }
    }

    fn apply(mut self, b: StyleUnit) -> CalcData {
        match b {
            StyleUnit::Unspecified => {}
            StyleUnit::Pixel(pixel) => self.pixel += pixel,
            StyleUnit::Point(point) => self.point += point,
            StyleUnit::Percent(percent) => self.percent += percent,
            StyleUnit::SlideWidthRatio(slide_width) => self.slide_width += slide_width,
            StyleUnit::SlideHeightRatio(slide_height) => self.slide_height += slide_height,
            StyleUnit::Calc(calc_data) => todo!(),
        }
        self
    }
}

impl Display for CalcData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "calc(")?;
        if self.pixel != 0.0 {
            write!(f, "+ {:}px", self.pixel)?;
        }
        if self.percent != 0.0 {
            write!(f, "+ {:}%", self.percent)?;
        }
        if self.point != 0.0 {
            write!(f, "+ {:}pt", self.point)?;
        }
        if self.slide_width != 0.0 {
            write!(f, "+ ({:} * var(--slide-width))", self.slide_width)?;
        }
        if self.slide_height != 0.0 {
            write!(f, "+ ({:} * var(--slide-height))", self.slide_height)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(Debug, strum::Display, Clone, Copy, PartialEq, Default)]
pub enum StyleUnit {
    #[strum(to_string = "unset")]
    #[default]
    Unspecified,
    #[strum(to_string = "{0}px")]
    Pixel(f64),
    #[strum(to_string = "{0}pt")]
    Point(f64),
    #[strum(to_string = "{0}%")]
    Percent(f64),
    #[strum(to_string = "calc({0} * var(--slide-width))")]
    SlideWidthRatio(f64),
    #[strum(to_string = "calc({0} * var(--slide-height))")]
    SlideHeightRatio(f64),
    #[strum(to_string = "{0}")]
    Calc(CalcData),
}

impl StyleUnit {
    fn add_pixel(&self, px: f64) -> StyleUnit {
        match self {
            StyleUnit::Unspecified => StyleUnit::Pixel(px),
            StyleUnit::Pixel(spx) => StyleUnit::Pixel(spx + px),
            StyleUnit::Point(pt) => todo!(),
            StyleUnit::Percent(percent) => todo!(),
            StyleUnit::SlideWidthRatio(_) => todo!(),
            StyleUnit::SlideHeightRatio(_) => todo!(),
            StyleUnit::Calc(calc_data) => StyleUnit::Calc(calc_data.add_pixel(px)),
        }
    }

    fn add_percent(&self, percent: f64) -> StyleUnit {
        match self {
            StyleUnit::Unspecified => StyleUnit::Percent(percent),
            StyleUnit::Pixel(_) => todo!(),
            StyleUnit::Point(pt) => todo!(),
            StyleUnit::Percent(spercent) => Self::Percent(*spercent + percent),
            StyleUnit::SlideWidthRatio(_) => todo!(),
            StyleUnit::SlideHeightRatio(_) => todo!(),
            StyleUnit::Calc(calc_data) => StyleUnit::Calc(calc_data.add_percent(percent)),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        match (self, &other) {
            (StyleUnit::Unspecified, max) => *max,
            (max, StyleUnit::Unspecified) => *max,
            (StyleUnit::Pixel(a), StyleUnit::Pixel(b)) => StyleUnit::Pixel(a.max(*b)),
            (StyleUnit::Point(a), StyleUnit::Point(b)) => StyleUnit::Point(a.max(*b)),
            (StyleUnit::Percent(a), StyleUnit::Percent(b)) => StyleUnit::Percent(a.max(*b)),
            _ => todo!(),
        }
    }

    pub fn or_zero(&self) -> StyleUnit {
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
            StyleUnit::Percent(percent) => self.add_percent(percent),
            StyleUnit::SlideWidthRatio(_) => todo!(),
            StyleUnit::SlideHeightRatio(_) => todo!(),
            StyleUnit::Calc(calc_data) => match self {
                StyleUnit::Unspecified => rhs,
                StyleUnit::Pixel(px) => StyleUnit::Calc(calc_data.add_pixel(px)),
                StyleUnit::Point(_) => todo!(),
                StyleUnit::Percent(percent) => StyleUnit::Calc(calc_data.add_percent(percent)),
                StyleUnit::SlideWidthRatio(_) => todo!(),
                StyleUnit::SlideHeightRatio(_) => todo!(),
                StyleUnit::Calc(calc_data) => todo!(),
            },
        }
    }
}

impl Sub for StyleUnit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs {
            StyleUnit::Unspecified => self,
            StyleUnit::Pixel(px) => self.add_pixel(-px),
            StyleUnit::Point(pt) => todo!(),
            StyleUnit::Percent(percent) => self.add_percent(-percent),
            StyleUnit::SlideWidthRatio(it) => {
                StyleUnit::Calc(CalcData::from_units(self, StyleUnit::SlideWidthRatio(-it)))
            }
            StyleUnit::SlideHeightRatio(it) => {
                StyleUnit::Calc(CalcData::from_units(self, StyleUnit::SlideHeightRatio(-it)))
            }
            StyleUnit::Calc(calc_data) => match self {
                StyleUnit::Unspecified => rhs,
                StyleUnit::Pixel(px) => todo!(),
                StyleUnit::Point(_) => todo!(),
                StyleUnit::Percent(percent) => todo!(),
                StyleUnit::SlideWidthRatio(_) => todo!(),
                StyleUnit::SlideHeightRatio(_) => todo!(),
                StyleUnit::Calc(calc_data) => todo!(),
            },
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
            "sw" => StyleUnit::SlideWidthRatio(number),
            "sh" => StyleUnit::SlideHeightRatio(number),
            _ => return Err(StyleUnitParseError::UnknownUnits),
        })
    }
}
