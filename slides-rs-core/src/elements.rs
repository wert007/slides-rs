use std::collections::HashSet;

use enum_dispatch::enum_dispatch;

use crate::{BaseElementStyling, Positioning, Result, output::PresentationEmitter};

mod image;
pub use image::*;
mod label;
pub use label::*;
mod custom_element;
pub use custom_element::*;

#[enum_dispatch]
pub trait WebRenderable {
    fn output_to_html<W: std::io::Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()>;
    fn collect_google_font_references(&self, _: &mut HashSet<String>) -> Result<()> {
        Ok(())
    }
    fn set_fallback_id(&mut self, id: String);
    fn set_id(&mut self, id: String);
    fn set_parent_id(&mut self, id: String);
    fn set_z_index(&mut self, z_index: usize);
}

#[enum_dispatch(WebRenderable)]
#[derive(Debug, Clone)]
pub enum Element {
    Image,
    Label,
    CustomElement,
}

// #[enum_dispatch(WebRenderable)]
#[derive(Debug)]
pub enum ElementRefMut<'a> {
    Image(&'a mut Image),
    Label(&'a mut Label),
    CustomElement(&'a mut CustomElement),
}

impl<'a> ElementRefMut<'a> {
    pub fn positioning_mut(&mut self) -> &mut Positioning {
        match self {
            ElementRefMut::Image(image) => image.positioning_mut(),
            ElementRefMut::Label(label) => label.positioning_mut(),
            ElementRefMut::CustomElement(custom_element) => custom_element.positioning_mut(),
        }
    }

    pub fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        match self {
            ElementRefMut::Image(image) => image.element_styling_mut().base_mut(),
            ElementRefMut::Label(label) => label.element_styling_mut().base_mut(),
            ElementRefMut::CustomElement(custom_element) => {
                custom_element.element_styling_mut().base_mut()
            }
        }
    }
}
