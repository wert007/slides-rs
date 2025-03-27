use enum_dispatch::enum_dispatch;

use crate::{Result, output::PresentationEmitter};

mod image;
pub use image::*;
mod label;
pub use label::*;

#[enum_dispatch]
pub trait WebRenderable {
    fn output_to_html<W: std::io::Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()>;
    fn set_fallback_id(&mut self, id: String);

    fn set_parent_id(&mut self, id: String);
}

#[enum_dispatch(WebRenderable)]
#[derive(Debug)]
pub enum Element {
    Image,
    Label,
}
