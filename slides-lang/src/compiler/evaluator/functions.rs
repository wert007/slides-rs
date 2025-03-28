use std::path::PathBuf;

use slides_rs_core::{Color, Image, ImageSource};

pub fn rgb(r: i64, g: i64, b: i64) -> Color {
    Color::rgb(r as _, g as _, b as _)
}

pub fn image(path: PathBuf) -> Image {
    Image::new(ImageSource::Path(path))
}
