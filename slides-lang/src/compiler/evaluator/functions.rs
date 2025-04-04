use std::path::PathBuf;

use slides_rs_core::{Color, Filter, Font, Image, ImageSource, Label};

pub fn rgb(r: i64, g: i64, b: i64) -> Color {
    Color::rgb(r as _, g as _, b as _)
}

pub fn image(path: PathBuf) -> Image {
    Image::new(ImageSource::Path(path))
}

pub fn label(text: String) -> Label {
    Label::new(text)
}

pub fn gfont(name: String) -> Font {
    Font::GoogleFont(name)
}

pub fn brightness(value: f64) -> Filter {
    Filter::Brightness(value)
}
