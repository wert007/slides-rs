use std::{collections::HashSet, path::PathBuf, sync::Arc};

use slides_rs_core::{
    BaseElementStyling, Color, Element, ElementId, Filter, Flex, Font, Grid, GridCellSize, Image,
    ImageSource, Label, Position, PresentationEmitter, StyleUnit, WebRenderable,
    WebRenderableContext,
    animations::{Animation, AnimationValue, Trigger},
};

use super::Evaluator;

pub fn rgb(r: i64, g: i64, b: i64) -> Color {
    Color::rgb(r as _, g as _, b as _)
}

pub fn image(path: PathBuf) -> Image {
    Image::new(ImageSource::Path(path))
}

pub fn label(text: String) -> Label {
    Label::new(text)
}

pub fn grid(columns: String, rows: String) -> Grid {
    fn parse_grid_cell_size(text: &str) -> GridCellSize {
        let (index, number) = text
            .char_indices()
            .take_while(|(_, c)| c.is_digit(10))
            .map(|(i, c)| {
                (
                    i + 1,
                    c.to_digit(10).expect("Filtered digits before") as usize,
                )
            })
            .reduce(|acc, val| {
                let number = acc.1 * 10 + val.1;
                (val.0, number)
            })
            .unwrap_or_else(|| (0, 1));

        match &text[index..] {
            "*" => GridCellSize::Fraction(number),
            "min" => GridCellSize::Minimum,
            unknown => todo!("Unknown unit {unknown}"),
        }
    }
    let columns = columns.split(',').map(parse_grid_cell_size).collect();
    let rows = rows.split(',').map(parse_grid_cell_size).collect();
    Grid::new(columns, rows)
}

pub fn gfont(name: String) -> Font {
    Font::GoogleFont(name)
}

pub fn brightness(value: f64) -> Filter {
    Filter::Brightness(value)
}

pub fn string(value: i64) -> String {
    value.to_string()
}

type StringArray = Vec<String>;

pub fn concat(value: StringArray) -> String {
    value.join("")
}

pub fn stackv(elements: Vec<Element>) -> Flex {
    let mut result = Flex::new(elements);
    result
        .styling_mut()
        .set_direction(slides_rs_core::FlexDirection::Column);
    result
}

pub fn stackh(elements: Vec<Element>) -> Flex {
    let mut result = Flex::new(elements);
    result
        .styling_mut()
        .set_direction(slides_rs_core::FlexDirection::Row);
    result
}

#[allow(non_snake_case)]
pub fn showAfterStep(step: i64) -> Animation {
    Animation {
        trigger: Trigger::StepReached(step as _),
        value: AnimationValue::ClassRemoval("invisible".into()),
    }
}

pub enum OwnedElement {
    Image(Image),
    Label(Label),
    CustomElement(slides_rs_core::CustomElement),
    Grid(Grid),
    Flex(Flex),
}

impl WebRenderable for OwnedElement {
    #[inline]
    fn output_to_html<W: std::io::Write>(
        self,
        emitter: &mut PresentationEmitter<W>,
        ctx: WebRenderableContext,
    ) -> slides_rs_core::Result<()> {
        match self {
            OwnedElement::Image(inner) => WebRenderable::output_to_html::<W>(inner, emitter, ctx),
            OwnedElement::Label(inner) => WebRenderable::output_to_html::<W>(inner, emitter, ctx),
            OwnedElement::CustomElement(inner) => {
                WebRenderable::output_to_html::<W>(inner, emitter, ctx)
            }
            OwnedElement::Grid(inner) => WebRenderable::output_to_html::<W>(inner, emitter, ctx),
            OwnedElement::Flex(inner) => WebRenderable::output_to_html::<W>(inner, emitter, ctx),
        }
    }
    #[inline]
    fn collect_google_font_references(
        &self,
        fonts: &mut HashSet<String>,
    ) -> slides_rs_core::Result<()> {
        match self {
            OwnedElement::Image(inner) => {
                WebRenderable::collect_google_font_references(inner, fonts)
            }
            OwnedElement::Label(inner) => {
                WebRenderable::collect_google_font_references(inner, fonts)
            }
            OwnedElement::CustomElement(inner) => {
                WebRenderable::collect_google_font_references(inner, fonts)
            }
            OwnedElement::Grid(inner) => {
                WebRenderable::collect_google_font_references(inner, fonts)
            }
            OwnedElement::Flex(inner) => {
                WebRenderable::collect_google_font_references(inner, fonts)
            }
        }
    }
    #[inline]
    fn set_parent(&mut self, parent: ElementId) {
        match self {
            OwnedElement::Image(inner) => WebRenderable::set_parent(inner, parent),
            OwnedElement::Label(inner) => WebRenderable::set_parent(inner, parent),
            OwnedElement::CustomElement(inner) => WebRenderable::set_parent(inner, parent),
            OwnedElement::Grid(inner) => WebRenderable::set_parent(inner, parent),
            OwnedElement::Flex(inner) => WebRenderable::set_parent(inner, parent),
        }
    }
    #[inline]
    fn parent(&self) -> Option<ElementId> {
        match self {
            OwnedElement::Image(inner) => WebRenderable::parent(inner),
            OwnedElement::Label(inner) => WebRenderable::parent(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::parent(inner),
            OwnedElement::Grid(inner) => WebRenderable::parent(inner),
            OwnedElement::Flex(inner) => WebRenderable::parent(inner),
        }
    }
    #[inline]
    fn id(&self) -> ElementId {
        match self {
            OwnedElement::Image(inner) => WebRenderable::id(inner),
            OwnedElement::Label(inner) => WebRenderable::id(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::id(inner),
            OwnedElement::Grid(inner) => WebRenderable::id(inner),
            OwnedElement::Flex(inner) => WebRenderable::id(inner),
        }
    }

    #[inline]
    fn name(&self) -> String {
        match self {
            OwnedElement::Image(inner) => WebRenderable::name(inner),
            OwnedElement::Label(inner) => WebRenderable::name(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::name(inner),
            OwnedElement::Grid(inner) => WebRenderable::name(inner),
            OwnedElement::Flex(inner) => WebRenderable::name(inner),
        }
    }

    #[inline]
    fn set_name(&mut self, name: String) {
        match self {
            OwnedElement::Image(inner) => WebRenderable::set_name(inner, name),
            OwnedElement::Label(inner) => WebRenderable::set_name(inner, name),
            OwnedElement::CustomElement(inner) => WebRenderable::set_name(inner, name),
            OwnedElement::Grid(inner) => WebRenderable::set_name(inner, name),
            OwnedElement::Flex(inner) => WebRenderable::set_name(inner, name),
        }
    }

    #[inline]
    fn namespace(&self) -> String {
        match self {
            OwnedElement::Image(inner) => WebRenderable::namespace(inner),
            OwnedElement::Label(inner) => WebRenderable::namespace(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::namespace(inner),
            OwnedElement::Grid(inner) => WebRenderable::namespace(inner),
            OwnedElement::Flex(inner) => WebRenderable::namespace(inner),
        }
    }

    #[inline]
    fn set_namespace(&mut self, namespace: String) {
        match self {
            OwnedElement::Image(inner) => WebRenderable::set_namespace(inner, namespace),
            OwnedElement::Label(inner) => WebRenderable::set_namespace(inner, namespace),
            OwnedElement::CustomElement(inner) => WebRenderable::set_namespace(inner, namespace),
            OwnedElement::Grid(inner) => WebRenderable::set_namespace(inner, namespace),
            OwnedElement::Flex(inner) => WebRenderable::set_namespace(inner, namespace),
        }
    }
    #[inline]
    fn element_styling(&self) -> &BaseElementStyling {
        match self {
            OwnedElement::Image(inner) => WebRenderable::element_styling(inner),
            OwnedElement::Label(inner) => WebRenderable::element_styling(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::element_styling(inner),
            OwnedElement::Grid(inner) => WebRenderable::element_styling(inner),
            OwnedElement::Flex(inner) => WebRenderable::element_styling(inner),
        }
    }
    #[inline]
    fn element_styling_mut(&mut self) -> &mut BaseElementStyling {
        match self {
            OwnedElement::Image(inner) => WebRenderable::element_styling_mut(inner),
            OwnedElement::Label(inner) => WebRenderable::element_styling_mut(inner),
            OwnedElement::CustomElement(inner) => WebRenderable::element_styling_mut(inner),
            OwnedElement::Grid(inner) => WebRenderable::element_styling_mut(inner),
            OwnedElement::Flex(inner) => WebRenderable::element_styling_mut(inner),
        }
    }
    #[inline]
    fn set_z_index(&mut self, z_index: usize) {
        match self {
            OwnedElement::Image(inner) => WebRenderable::set_z_index(inner, z_index),
            OwnedElement::Label(inner) => WebRenderable::set_z_index(inner, z_index),
            OwnedElement::CustomElement(inner) => WebRenderable::set_z_index(inner, z_index),
            OwnedElement::Grid(inner) => WebRenderable::set_z_index(inner, z_index),
            OwnedElement::Flex(inner) => WebRenderable::set_z_index(inner, z_index),
        }
    }
}

impl From<Element> for OwnedElement {
    fn from(value: Element) -> Self {
        match value {
            Element::Image(it) => Self::Image(it.get_cloned().unwrap()),
            Element::Label(it) => OwnedElement::Label(it.get_cloned().unwrap()),
            Element::CustomElement(it) => OwnedElement::CustomElement(it.get_cloned().unwrap()),
            Element::Grid(it) => OwnedElement::Grid(it.get_cloned().unwrap()),
            Element::Flex(it) => OwnedElement::Flex(it.get_cloned().unwrap()),
            Element::Element(it) => it.get_cloned().unwrap().into(),
        }
    }
}

// impl WebRenderable for OwnedElement {

// }

#[allow(non_snake_case)]
pub fn leftTop(_evaluator: &mut Evaluator, element: Element) -> Position {
    let evaluator = _evaluator;
    let parent = element.parent();
    let (x_offset, y_offset) = match parent {
        Some(_parent) => todo!(),
        None => (StyleUnit::Unspecified, StyleUnit::Unspecified),
    };
    let slide_padding = evaluator
        .slide
        .as_mut()
        .unwrap()
        .styling_mut()
        .base()
        .padding;
    let element = OwnedElement::from(element);

    let x = match element.element_styling().halign {
        slides_rs_core::HorizontalAlignment::Unset
        | slides_rs_core::HorizontalAlignment::Stretch
        | slides_rs_core::HorizontalAlignment::Left => {
            element.element_styling().margin.left + slide_padding.left
        }
        slides_rs_core::HorizontalAlignment::Center => todo!(),
        slides_rs_core::HorizontalAlignment::Right => todo!(),
    };
    let y = match element.element_styling().valign {
        slides_rs_core::VerticalAlignment::Unset
        | slides_rs_core::VerticalAlignment::Stretch
        | slides_rs_core::VerticalAlignment::Top => {
            element.element_styling().margin.top + slide_padding.top
        }
        slides_rs_core::VerticalAlignment::Center => todo!(),
        slides_rs_core::VerticalAlignment::Bottom => todo!(),
    };
    let x = x_offset + x;
    let y = y_offset + y;
    Position { x, y }
}

#[allow(non_snake_case)]
pub fn sizeOf(_evaluator: &mut Evaluator, element: Element) -> Position {
    let evaluator = _evaluator;
    let parent = element.parent();
    let (x_offset, y_offset) = match parent {
        Some(_parent) => todo!(),
        None => (StyleUnit::Unspecified, StyleUnit::Unspecified),
    };
    let slide_padding = evaluator
        .slide
        .as_mut()
        .unwrap()
        .styling_mut()
        .base()
        .padding;
    let element = OwnedElement::from(element);

    let x = match element.element_styling().halign {
        slides_rs_core::HorizontalAlignment::Unset | slides_rs_core::HorizontalAlignment::Left => {
            todo!()
        }
        slides_rs_core::HorizontalAlignment::Stretch => {
            x_offset.min(StyleUnit::SlideWidthRatio(1.0)) - slide_padding.left - slide_padding.right
        }
        slides_rs_core::HorizontalAlignment::Center => todo!(),
        slides_rs_core::HorizontalAlignment::Right => todo!(),
    };
    let y = match element.element_styling().valign {
        slides_rs_core::VerticalAlignment::Unset | slides_rs_core::VerticalAlignment::Top => {
            todo!()
        }
        slides_rs_core::VerticalAlignment::Stretch => {
            y_offset.min(StyleUnit::SlideHeightRatio(1.0))
                - slide_padding.top
                - slide_padding.bottom
        }
        slides_rs_core::VerticalAlignment::Center => todo!(),
        slides_rs_core::VerticalAlignment::Bottom => todo!(),
    };
    Position { x, y }
}

#[allow(non_snake_case)]
pub fn positionInside(_evaluator: &mut Evaluator, element: Element, x: f64, y: f64) -> Position {
    let evaluator = _evaluator;
    let size = sizeOf(evaluator, element.clone());
    let left_top = leftTop(evaluator, element.clone());
    let x = left_top.x + size.x * x;
    let y = left_top.y + size.y * y;
    Position { x, y }
}
