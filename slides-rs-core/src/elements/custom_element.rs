use std::io::Write;

use crate::{ElementStyling, Result, StylingReference, ToCss, output::PresentationEmitter};

use super::{Element, WebRenderable};

#[derive(Debug, Clone)]
pub struct CustomElement {
    parent_id: String,
    id: String,
    styling: ElementStyling<()>,
    stylings: Vec<StylingReference>,
    type_name: String,
    children: Vec<Element>,
}
impl CustomElement {
    pub fn new(type_name: impl Into<String>, children: Vec<Element>) -> Self {
        Self {
            parent_id: String::new(),
            id: String::new(),
            type_name: type_name.into(),
            children,
            styling: ElementStyling::new_base(),
            stylings: Vec::new(),
        }
    }
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn as_element_mut(&mut self) -> super::ElementRefMut<'_> {
        super::ElementRefMut::CustomElement(self)
    }

    pub(crate) fn element_styling_mut(&mut self) -> &mut ElementStyling<()> {
        &mut self.styling
    }

    pub fn with_element_styling(mut self, styling: ElementStyling<()>) -> Self {
        self.styling = styling;
        self
    }

    pub fn add_styling(&mut self, reference: StylingReference) {
        self.stylings.push(reference);
    }
}

impl WebRenderable for CustomElement {
    fn output_to_html<W: Write>(self, emitter: &mut PresentationEmitter<W>) -> Result<()> {
        let id = format!("{}-{}", self.parent_id, self.id);
        let style = self.styling.to_css_style();
        writeln!(
            emitter.raw_css(),
            r#"#{id} {{
            {style}
        }}"#,
        )?;
        writeln!(
            emitter.raw_html(),
            "<div id=\"{id}\" class=\"custom-element {} {}\">",
            self.type_name,
            self.stylings
                .into_iter()
                .map(|s| format!(" {s}"))
                .collect::<String>(),
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

    fn element_styling_mut(&mut self) -> &mut crate::BaseElementStyling {
        self.element_styling_mut().base_mut()
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
