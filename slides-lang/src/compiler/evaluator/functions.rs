use std::path::PathBuf;

use slides_rs_core::{
    Color, Element, Filter, Flex, Font, Grid, GridCellSize, Image, ImageSource, Label,
    animations::{Animation, AnimationValue, Trigger},
};

pub fn rgb(r: i64, g: i64, b: i64) -> Color {
    Color::rgb(r as _, g as _, b as _)
}

pub fn image(path: PathBuf) -> Image {
    Image::new(ImageSource::Path(path))
}

pub fn label(text: String) -> Label {
    Label::new(text)
}

pub fn grid(columns: String, rows: String) -> Grid {
    fn parse_grid_cell_size(text: &str) -> GridCellSize {
        let (index, number) = text
            .char_indices()
            .take_while(|(_, c)| c.is_digit(10))
            .map(|(i, c)| {
                (
                    i + 1,
                    c.to_digit(10).expect("Filtered digits before") as usize,
                )
            })
            .reduce(|acc, val| {
                let number = acc.1 * 10 + val.1;
                (val.0, number)
            })
            .unwrap_or_else(|| (0, 1));

        match &text[index..] {
            "*" => GridCellSize::Fraction(number),
            "min" => GridCellSize::Minimum,
            unknown => todo!("Unknown unit {unknown}"),
        }
    }
    let columns = columns.split(',').map(parse_grid_cell_size).collect();
    let rows = rows.split(',').map(parse_grid_cell_size).collect();
    Grid::new(columns, rows)
}

pub fn gfont(name: String) -> Font {
    Font::GoogleFont(name)
}

pub fn brightness(value: f64) -> Filter {
    Filter::Brightness(value)
}

pub fn string(value: i64) -> String {
    value.to_string()
}

type StringArray = Vec<String>;

pub fn concat(value: StringArray) -> String {
    value.join("")
}

pub fn stackv(elements: Vec<Element>) -> Flex {
    let mut result = Flex::new(elements);
    result
        .styling_mut()
        .set_direction(slides_rs_core::FlexDirection::Column);
    result
}

pub fn stackh(elements: Vec<Element>) -> Flex {
    let mut result = Flex::new(elements);
    result
        .styling_mut()
        .set_direction(slides_rs_core::FlexDirection::Row);
    result
}

#[allow(non_snake_case)]
pub fn showAfterStep(step: i64) -> Animation {
    Animation {
        trigger: Trigger::StepReached(step as _),
        value: AnimationValue::ClassRemoval("invisible".into()),
    }
}
