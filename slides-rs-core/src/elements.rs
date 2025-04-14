use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::Display,
    sync::{Arc, Once, OnceLock, atomic::AtomicUsize},
};

use enum_dispatch::enum_dispatch;

use crate::{
    BaseElementStyling, Result, StyleUnit, StylingReference, ToCssLayout,
    output::PresentationEmitter,
};

mod image;
pub use image::*;
mod label;
pub use label::*;
mod custom_element;
pub use custom_element::*;
mod grid;
pub use grid::*;

#[derive(Debug, Clone, Copy)]
pub struct ElementId(usize);

static NEXT_ELEMENT_INDEX: AtomicUsize = AtomicUsize::new(0);

impl ElementId {
    pub fn generate() -> Self {
        let raw = NEXT_ELEMENT_INDEX.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        ElementId(raw)
    }
}

impl Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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
    fn set_parent(&mut self, parent: ElementId);
    fn parent(&self) -> Option<ElementId>;
    fn id(&self) -> ElementId;
    fn set_name(&mut self, name: String);
    fn set_namespace(&mut self, namespace: String);
    fn element_styling(&self) -> &BaseElementStyling;
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
    Grid,
}

// #[enum_dispatch(WebRenderable)]
#[derive(Debug)]
pub enum ElementRefMut {
    Image(Arc<RefCell<Image>>),
    Label(Arc<RefCell<Label>>),
    CustomElement(Arc<RefCell<CustomElement>>),
    Grid(Arc<RefCell<Grid>>),
}

impl ElementRefMut {
    pub fn has_parent(&self) -> bool {
        match self {
            ElementRefMut::Image(it) => it.borrow().parent().is_some(),
            ElementRefMut::Label(it) => it.borrow().parent().is_some(),
            ElementRefMut::CustomElement(it) => it.borrow().parent().is_some(),
            ElementRefMut::Grid(it) => it.borrow().parent().is_some(),
        }
    }

    pub fn apply_to_base_element_styling(&mut self, mut cb: impl FnMut(&mut BaseElementStyling)) {
        match self {
            ElementRefMut::Image(it) => cb(it.borrow_mut().element_styling_mut().base_mut()),
            ElementRefMut::Label(it) => cb(it.borrow_mut().element_styling_mut().base_mut()),
            ElementRefMut::CustomElement(it) => {
                cb(it.borrow_mut().element_styling_mut().base_mut())
            }
            ElementRefMut::Grid(it) => cb(it.borrow_mut().element_styling_mut()),
        }
    }

    pub fn add_styling_reference(&mut self, reference: StylingReference) {
        match self {
            ElementRefMut::Image(image) => image.borrow_mut().add_styling(reference),
            ElementRefMut::Label(label) => label.borrow_mut().add_styling(reference),
            ElementRefMut::CustomElement(custom_element) => {
                custom_element.borrow_mut().add_styling(reference)
            }
            ElementRefMut::Grid(it) => it.borrow_mut().add_styling(reference),
        }
    }

    pub fn set_vertical_alignment(&mut self, value: crate::VerticalAlignment) {
        self.apply_to_base_element_styling(|base| base.set_vertical_alignment(value));
    }

    pub fn set_width(&mut self, value: StyleUnit) {
        self.apply_to_base_element_styling(|base| base.set_width(value));
    }

    pub fn set_height(&mut self, value: StyleUnit) {
        self.apply_to_base_element_styling(|base| base.set_height(value));
    }

    pub fn set_z_index(&mut self, value: usize) {
        self.apply_to_base_element_styling(|base| base.set_z_index(value));
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
