use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use enum_dispatch::enum_dispatch;

use crate::{
    BaseElementStyling, Result, StylingReference, ToCssLayout, output::PresentationEmitter,
};

mod image;
pub use image::*;
mod label;
pub use label::*;
mod custom_element;
pub use custom_element::*;

#[derive(Debug, Clone, Copy)]
pub struct WebRenderableContext {
    pub layout: ToCssLayout,
}

#[enum_dispatch]
pub trait WebRenderable {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()>;
    fn collect_google_font_references(&self, _: &mut HashSet<String>) -> Result<()> {
        Ok(())
    }
    fn set_fallback_id(&mut self, id: String);
    fn set_id(&mut self, id: String);
    fn set_parent_id(&mut self, id: String);
    fn element_styling_mut(&mut self) -> &mut BaseElementStyling;
    fn set_z_index(&mut self, z_index: usize) {
        self.element_styling_mut().set_z_index(z_index)
    }
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
pub enum ElementRefMut {
    Image(Arc<RefCell<Image>>),
    Label(Arc<RefCell<Label>>),
    CustomElement(Arc<RefCell<CustomElement>>),
}

impl ElementRefMut {
    // pub fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
    //     match self {
    //         ElementRefMut::Image(image) => image.borrow_mut().element_styling_mut().base_mut(),
    //         ElementRefMut::Label(label) => label.borrow_mut().element_styling_mut().base_mut(),
    //         ElementRefMut::CustomElement(custom_element) => {
    //             custom_element.borrow_mut().element_styling_mut().base_mut()
    //         }
    //     }
    // }

    pub fn apply_to_base_element_styling(&mut self, mut cb: impl FnMut(&mut BaseElementStyling)) {
        match self {
            ElementRefMut::Image(it) => cb(it.borrow_mut().element_styling_mut().base_mut()),
            ElementRefMut::Label(it) => cb(it.borrow_mut().element_styling_mut().base_mut()),
            ElementRefMut::CustomElement(it) => {
                cb(it.borrow_mut().element_styling_mut().base_mut())
            }
        }
    }

    pub fn add_styling_reference(&mut self, reference: StylingReference) {
        match self {
            ElementRefMut::Image(image) => image.borrow_mut().add_styling(reference),
            ElementRefMut::Label(label) => label.borrow_mut().add_styling(reference),
            ElementRefMut::CustomElement(custom_element) => {
                custom_element.borrow_mut().add_styling(reference)
            }
        }
    }

    pub fn set_vertical_alignment(&mut self, value: crate::VerticalAlignment) {
        self.apply_to_base_element_styling(|base| base.set_vertical_alignment(value));
    }

    pub fn set_horizontal_alignment(&mut self, value: crate::HorizontalAlignment) {
        self.apply_to_base_element_styling(|base| base.set_horizontal_alignment(value));
    }

    pub fn set_margin(&mut self, value: crate::Thickness) {
        self.apply_to_base_element_styling(|base| base.set_margin(value));
    }

    pub fn set_padding(&mut self, value: crate::Thickness) {
        self.apply_to_base_element_styling(|base| base.set_padding(value));
    }

    pub fn set_background(&mut self, value: crate::Background) {
        self.apply_to_base_element_styling(|base| base.set_background(value));
    }

    pub fn set_filter(&mut self, value: crate::Filter) {
        self.apply_to_base_element_styling(|base| base.set_filter(value));
    }
}
