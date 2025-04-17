use std::{cell::RefCell, sync::Arc};

use struct_field_names_as_array::FieldNamesAsSlice;

use crate::{
    BaseElementStyling, ElementStyling, GridCellSize, GridStyling, Result, StyleUnit,
    StylingReference, ToCss, output::PresentationEmitter,
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
    element_grid_data: Vec<Arc<RefCell<GridEntry>>>,
    // text: FormattedText,
    styling: ElementStyling<GridStyling>,
    stylings: Vec<StylingReference>,
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
        }
    }

    pub fn add_styling(&mut self, styling: StylingReference) {
        self.stylings.push(styling);
    }

    // This is a weird api choice, but it is necessary for the language to work.
    pub fn add_element(&mut self, element: Element) -> Arc<RefCell<GridEntry>> {
        let entry = Arc::new(RefCell::new(GridEntry::new()));
        // element.set
        self.children.push(element);
        self.element_grid_data.push(entry.clone());
        entry
    }

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            format!("{}-{}", self.element_styling().class_name(), self.id)
        } else {
            self.name.clone()
        }
    }
}

impl WebRenderable for Grid {
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        mut ctx: WebRenderableContext,
    ) -> Result<()> {
        let id = format!("{}-{}", self.namespace, self.name());
        self.styling
            .to_css_rule(ctx.layout, &format!("#{id}"), emitter.raw_css())?;
        writeln!(emitter.raw_html(), "<div id=\"{id}\" class=\"grid\">")?;
        for (index, (mut element, data)) in self
            .children
            .into_iter()
            .zip(self.element_grid_data)
            .enumerate()
        {
            let grid_data = Arc::unwrap_or_clone(data).into_inner();
            element.set_namespace(id.clone());
            // element.set_fallback_id(index.to_string());

            ctx.layout.grid_data = Some(grid_data);
            element.output_to_html(emitter, ctx)?;
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

    fn set_name(&mut self, id: String) {
        self.name = id;
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
