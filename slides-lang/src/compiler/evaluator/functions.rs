use std::path::PathBuf;

use slides_rs_core::{Color, Filter, Font, Grid, GridCellSize, Image, ImageSource, Label};

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
        match text {
            "*" => GridCellSize::Fraction(1),
            "min" => GridCellSize::Minimum,
            _ => todo!(),
        }
    }
    let columns = columns.split('|').map(parse_grid_cell_size).collect();
    let rows = rows.split('|').map(parse_grid_cell_size).collect();
    // TODO: Parse columns and rows and construct grid out of it!
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
