use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use struct_field_names_as_array::FieldNamesAsSlice;

use crate::{
    BaseElementStyling, ElementStyling, GridCellSize, GridStyling, Result, StylingReference, ToCss,
    animations::Animations, output::PresentationEmitter,
};

use super::{Element, ElementId, WebRenderable, WebRenderableContext};

#[derive(Debug, Clone, Copy, FieldNamesAsSlice)]
pub struct GridEntry {
    pub column_span: usize,
    pub row_span: usize,
}
impl GridEntry {
    fn new() -> Self {
        Self {
            column_span: 1,
            row_span: 1,
        }
    }
}

#[derive(Debug, Clone, FieldNamesAsSlice)]
pub struct Grid {
    namespace: String,
    name: String,
    id: ElementId,
    parent: Option<ElementId>,
    children: Vec<Element>,
    element_grid_data: Vec<Arc<RwLock<GridEntry>>>,
    styling: ElementStyling<GridStyling>,
    stylings: Vec<StylingReference>,
    pub animations: Animations,
}

impl Grid {
    pub fn new(columns: Vec<GridCellSize>, rows: Vec<GridCellSize>) -> Self {
        Self {
            namespace: String::new(),
            name: String::new(),
            id: ElementId::generate(),
            parent: None,
            children: Vec::new(),
            element_grid_data: Vec::new(),
            styling: GridStyling::new(columns, rows),
            stylings: Vec::new(),
            animations: Animations::new(),
        }
    }

    pub fn add_styling(&mut self, styling: StylingReference) {
        self.stylings.push(styling);
    }

    // This is a weird api choice, but it is necessary for the language to work.
    pub fn add_element(&mut self, element: Element) -> Arc<RwLock<GridEntry>> {
        let entry = Arc::new(RwLock::new(GridEntry::new()));
        // element.set
        self.children.push(element);
        self.element_grid_data.push(entry.clone());
        entry
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.styling.class_name(), self.id)
        } else {
            self.name.clone()
        }
    }
}

impl WebRenderable for Grid {
    fn output_to_html<W: std::io::Write>(
        mut self,
        emitter: &mut PresentationEmitter<W>,
        mut ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        let classes_animations = self.animations.get_initial_classes();
        let classes = self
            .stylings
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        self.animations
            .emit_to_javascript(emitter.raw_js(), ctx.clone(), &id)?;
        self.animations.apply_to_styling(&mut self.styling);
        self.styling
            .to_css_rule(ctx.layout.clone(), &format!("#{id}"), emitter.raw_css())?;
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"grid {classes} {classes_animations}\" data-element-id=\"{}\">",
            self.id.raw()
        )?;
        for (mut element, data) in self.children.into_iter().zip(self.element_grid_data) {
            let grid_data = Arc::try_unwrap(data).unwrap().into_inner().unwrap();
            element.set_namespace(id.clone());
            // element.set_fallback_id(index.to_string());

            ctx.layout.grid_data = Some(grid_data);
            element.output_to_html(emitter, ctx.clone())?;
        }
        writeln!(emitter.raw_html(), "</div>")?;

        Ok(())
    }

    fn set_parent(&mut self, parent: ElementId) {
        self.parent = Some(parent);
    }

    fn parent(&self) -> Option<ElementId> {
        self.parent
    }

    fn id(&self) -> ElementId {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, id: String) {
        self.name = id;
    }

    fn namespace(&self) -> String {
        self.namespace.clone()
    }

    fn set_namespace(&mut self, id: String) {
        self.namespace = id;
    }

    fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        self.styling.base_mut()
    }

    fn element_styling(&self) -> &BaseElementStyling {
        self.styling.base()
    }
}
