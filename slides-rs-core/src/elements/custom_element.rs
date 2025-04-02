use std::io::Write;

use crate::{ElementStyling, Positioning, Result, ToCss, output::PresentationEmitter};

use super::{Element, WebRenderable};

#[derive(Debug, Clone)]
pub struct CustomElement {
    z_index: usize,
    parent_id: String,
    id: String,
    styling: ElementStyling<()>,
    positioning: Positioning,
    type_name: String,
    children: Vec<Element>,
}
impl CustomElement {
    pub fn new(type_name: impl Into<String>, children: Vec<Element>) -> Self {
        Self {
            z_index: 0,
            parent_id: String::new(),
            id: String::new(),
            type_name: type_name.into(),
            children,
            styling: ElementStyling::new_base(),
            positioning: Positioning::new(),
        }
    }
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn as_element_mut(&mut self) -> super::ElementRefMut<'_> {
        super::ElementRefMut::CustomElement(self)
    }

    pub fn positioning_mut(&mut self) -> &mut Positioning {
        &mut self.positioning
    }

    pub(crate) fn element_styling_mut(&mut self) -> &mut ElementStyling<()> {
        &mut self.styling
    }
}

impl WebRenderable for CustomElement {
    fn output_to_html<W: Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        let id = format!("{}-{}", self.parent_id, self.id);
        let positioning = self.positioning.to_css_style();
        let style = self.styling.to_css_style();
        writeln!(
            emitter.raw_css(),
            r#"#{id} {{
            z-index: {};
            {positioning}
            {style}
        }}"#,
            self.z_index
        )?;
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"custom-element {}\">",
            self.type_name()
        )?;
        for element in self.children {
            element.output_to_html(emitter)?;
        }
        writeln!(emitter.raw_html(), "</div>")?;
        Ok(())
    }

    fn set_fallback_id(&mut self, id: String) {
        if self.id.is_empty() {
            self.id = id;
            self.children
                .iter_mut()
                .for_each(|c| c.set_parent_id(format!("{}-{}", self.parent_id, self.id)));
        }
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
        self.children
            .iter_mut()
            .for_each(|c| c.set_parent_id(format!("{}-{}", self.parent_id, self.id)));
    }

    fn set_parent_id(&mut self, id: String) {
        self.parent_id = id;
        self.children
            .iter_mut()
            .for_each(|c| c.set_parent_id(format!("{}-{}", self.parent_id, self.id)));
    }

    fn set_z_index(&mut self, z_index: usize) {
        self.z_index = z_index;
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> crate::Result<()> {
        for child in &self.children {
            child.collect_google_font_references(fonts)?;
        }
        Ok(())
    }
}
