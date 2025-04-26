use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::Display,
    sync::{Arc, RwLock, atomic::AtomicUsize},
};

use enum_dispatch::enum_dispatch;

use crate::{
    BaseElementStyling, Result, StyleUnit, StylingReference, ToCssLayout, animations::Animation,
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
mod flex;
pub use flex::*;

#[derive(Debug, Clone, Copy)]
pub struct ElementId(usize);

static NEXT_ELEMENT_INDEX: AtomicUsize = AtomicUsize::new(0);

impl ElementId {
    pub fn generate() -> Self {
        let raw = NEXT_ELEMENT_INDEX.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        ElementId(raw)
    }

    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct WebRenderableContext {
    pub layout: ToCssLayout,
    pub slide_name: String,
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
    fn name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn namespace(&self) -> String;
    fn set_namespace(&mut self, namespace: String);
    fn element_styling(&self) -> &BaseElementStyling;
    fn element_styling_mut(&mut self) -> &mut BaseElementStyling;
    fn set_z_index(&mut self, z_index: usize) {
        self.element_styling_mut().set_z_index(z_index)
    }
}

impl<T: WebRenderable + Clone> WebRenderable for Arc<RefCell<T>> {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()> {
        Arc::unwrap_or_clone(self)
            .into_inner()
            .output_to_html(emitter, ctx)
    }

    fn set_parent(&mut self, parent: ElementId) {
        self.borrow_mut().set_parent(parent);
    }

    fn parent(&self) -> Option<ElementId> {
        self.borrow().parent()
    }

    fn id(&self) -> ElementId {
        self.borrow().id()
    }

    fn name(&self) -> String {
        self.borrow().name()
    }

    fn set_name(&mut self, name: String) {
        self.borrow_mut().set_name(name);
    }

    fn namespace(&self) -> String {
        self.borrow().namespace()
    }

    fn set_namespace(&mut self, namespace: String) {
        self.borrow_mut().set_namespace(namespace);
    }

    fn element_styling(&self) -> &BaseElementStyling {
        todo!("How would we do that?")
        // self.borrow().element_styling()
    }

    fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        todo!("How would we do that?")
        // self.borrow_mut().element_styling_mut()
    }

    fn collect_google_font_references(&self, fonts: &mut HashSet<String>) -> Result<()> {
        self.borrow().collect_google_font_references(fonts)
    }

    fn set_z_index(&mut self, z_index: usize) {
        self.borrow_mut().set_z_index(z_index);
    }
}

impl<T: WebRenderable + Clone> WebRenderable for Arc<RwLock<T>> {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> Result<()> {
        self.get_cloned().unwrap().output_to_html(emitter, ctx)
    }

    fn set_parent(&mut self, parent: ElementId) {
        self.write().unwrap().set_parent(parent);
    }

    fn parent(&self) -> Option<ElementId> {
        self.read().unwrap().parent()
    }

    fn id(&self) -> ElementId {
        self.read().unwrap().id()
    }

    fn name(&self) -> String {
        self.read().unwrap().name()
    }

    fn set_name(&mut self, name: String) {
        self.write().unwrap().set_name(name);
    }

    fn namespace(&self) -> String {
        self.read().unwrap().namespace()
    }

    fn set_namespace(&mut self, namespace: String) {
        self.write().unwrap().set_namespace(namespace);
    }

    fn element_styling(&self) -> &BaseElementStyling {
        todo!("How would we do that?")
        // self.read().unwrap().element_styling()
    }

    fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        todo!("How would we do that?")
        // self.write().unwrap().element_styling_mut()
    }

    fn collect_google_font_references(&self, fonts: &mut HashSet<String>) -> Result<()> {
        self.read().unwrap().collect_google_font_references(fonts)
    }

    fn set_z_index(&mut self, z_index: usize) {
        self.write().unwrap().set_z_index(z_index);
    }
}

#[enum_dispatch(WebRenderable)]
#[derive(Debug, Clone)]
pub enum Element {
    Image(Arc<RwLock<Image>>),
    Label(Arc<RwLock<Label>>),
    CustomElement(Arc<RwLock<CustomElement>>),
    Grid(Arc<RwLock<Grid>>),
    Flex(Arc<RwLock<Flex>>),
    Element(Arc<RwLock<Element>>),
}

impl From<Label> for Element {
    fn from(value: Label) -> Self {
        Self::Label(Arc::new(RwLock::new(value)))
    }
}

impl From<Image> for Element {
    fn from(value: Image) -> Self {
        Self::Image(Arc::new(RwLock::new(value)))
    }
}

impl From<CustomElement> for Element {
    fn from(value: CustomElement) -> Self {
        Self::CustomElement(Arc::new(RwLock::new(value)))
    }
}

impl From<Grid> for Element {
    fn from(value: Grid) -> Self {
        Self::Grid(Arc::new(RwLock::new(value)))
    }
}

impl From<Flex> for Element {
    fn from(value: Flex) -> Self {
        Self::Flex(Arc::new(RwLock::new(value)))
    }
}

// #[enum_dispatch(WebRenderable)]
#[derive(Debug)]
pub enum ElementRefMut {
    Image(Arc<RwLock<Image>>),
    Label(Arc<RwLock<Label>>),
    CustomElement(Arc<RwLock<CustomElement>>),
    Grid(Arc<RwLock<Grid>>),
    Flex(Arc<RwLock<Flex>>),
}

impl ElementRefMut {
    pub fn has_parent(&self) -> bool {
        match self {
            ElementRefMut::Image(it) => it.read().unwrap().parent().is_some(),
            ElementRefMut::Label(it) => it.read().unwrap().parent().is_some(),
            ElementRefMut::CustomElement(it) => it.read().unwrap().parent().is_some(),
            ElementRefMut::Grid(it) => it.read().unwrap().parent().is_some(),
            ElementRefMut::Flex(it) => it.read().unwrap().parent().is_some(),
        }
    }

    pub fn apply_to_base_element_styling(&mut self, mut cb: impl FnMut(&mut BaseElementStyling)) {
        match self {
            ElementRefMut::Image(it) => cb(it.write().unwrap().element_styling_mut().base_mut()),
            ElementRefMut::Label(it) => cb(it.write().unwrap().element_styling_mut().base_mut()),
            ElementRefMut::CustomElement(it) => {
                cb(it.write().unwrap().element_styling_mut().base_mut())
            }
            ElementRefMut::Grid(it) => cb(it.write().unwrap().element_styling_mut()),
            ElementRefMut::Flex(it) => cb(it.write().unwrap().element_styling_mut()),
        }
    }

    pub fn add_styling_reference(&mut self, reference: StylingReference) {
        match self {
            ElementRefMut::Image(image) => image.write().unwrap().add_styling(reference),
            ElementRefMut::Label(label) => label.write().unwrap().add_styling(reference),
            ElementRefMut::CustomElement(custom_element) => {
                custom_element.write().unwrap().add_styling(reference)
            }
            ElementRefMut::Grid(it) => it.write().unwrap().add_styling(reference),
            ElementRefMut::Flex(it) => it.write().unwrap().add_styling(reference),
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

    pub fn set_animations(&mut self, value: Vec<Animation>) {
        match self {
            ElementRefMut::Image(it) => it.write().unwrap().animations.add_animations(&value),
            ElementRefMut::Label(it) => it.write().unwrap().animations.add_animations(&value),
            ElementRefMut::CustomElement(it) => {
                it.write().unwrap().animations.add_animations(&value)
            }
            ElementRefMut::Grid(it) => it.write().unwrap().animations.add_animations(&value),
            ElementRefMut::Flex(it) => it.write().unwrap().animations.add_animations(&value),
        }
    }
}
