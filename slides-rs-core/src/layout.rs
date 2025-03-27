use crate::Result;

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

    pub(crate) fn to_css_style(&self) -> Option<String> {
        use std::fmt::Write;
        let mut result = String::new();
        let mut translate = (0.0, 0.0);

        match self.vertical_alignment {
            VerticalAlignment::Top => {
                writeln!(result, "top: 0px;").expect("infallible");
            }
            VerticalAlignment::Center => {
                writeln!(result, "top: 50%;").expect("infallible");
                translate.1 = -50.0;
            }
            VerticalAlignment::Bottom => {
                writeln!(result, "bottom: 0px;").expect("infallible");
            }
            VerticalAlignment::Stretch => {
                writeln!(result, "top: 0px;\nbottom: 0px;\nheight: 100%;").expect("infallible");
            }
        }

        match self.horizontal_alignment {
            HorizontalAlignment::Left => {
                writeln!(result, "left: 0px;").expect("infallible");
            }
            HorizontalAlignment::Center => {
                writeln!(result, "left: 50%;").expect("infallible");
                translate.0 = -50.0;
            }
            HorizontalAlignment::Right => {
                writeln!(result, "right: 0px;").expect("infallible");
            }
            HorizontalAlignment::Stretch => {
                writeln!(result, "left: 0px;\nbottom: 0px;\nwidth: 100%;").expect("infallible");
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

#[derive(Debug)]
pub enum StyleUnit {
    Unspecified,
    Pixel(f64),
}
