use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use slides_rs_core::Element;
use summum_types::summum;

use crate::{
    VariableId,
    compiler::{
        binder::{
            BoundNode,
            typing::{Type, TypeId},
        },
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub id: VariableId,
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct UserFunctionValue {
    pub has_implicit_slide_parameter: bool,
    pub parameters: Vec<Parameter>,
    pub body: Vec<BoundNode>,
    pub return_type: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub struct None {}

summum! {
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    pub enum Value {
        Void(()),
        None(None),
        Float(f64),
        Integer(i64),
        String(String),
        StyleReference(slides_rs_core::StylingReference),
        Background(slides_rs_core::Background),
        Color(slides_rs_core::Color),
        Label(Arc<RwLock<slides_rs_core::Label>>),
        Grid(Arc<RwLock<slides_rs_core::Grid>>),
        Flex(Arc<RwLock<slides_rs_core::Flex>>),
        GridEntry(Arc<RwLock<slides_rs_core::GridEntry>>),
        Path(PathBuf),
        Image(Arc<RwLock<slides_rs_core::Image>>),
        ObjectFit(slides_rs_core::ObjectFit),
        VerticalAlignment(slides_rs_core::VerticalAlignment),
        HorizontalAlignment(slides_rs_core::HorizontalAlignment),
        TextAlign(slides_rs_core::TextAlign),
        Font(slides_rs_core::Font),
        StyleUnit(slides_rs_core::StyleUnit),
        Dict(HashMap<String, Value>),
        UserFunction(UserFunctionValue),
        CustomElement(Arc<RwLock<slides_rs_core::CustomElement>>),
        Thickness(slides_rs_core::Thickness),
        Array(Vec<Value>),
        Filter(slides_rs_core::Filter),
        Animation(slides_rs_core::animations::Animation),
        TextStyling(Arc<RwLock<slides_rs_core::TextStyling>>),
        Element(slides_rs_core::Element),
        Position(slides_rs_core::Position),
        Module(Arc<RwLock<Module>>),
    }
}

impl Value {
    pub fn none() -> Self {
        Value::None(None {})
    }

    pub fn infer_type(&self) -> Type {
        match self {
            Value::Void(()) => Type::Void,
            Value::None(_) => Type::None,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
            Value::StyleReference(_) => Type::Styling,
            Value::Background(_) => Type::Background,
            Value::Color(_) => Type::Color,
            Value::Label(_) => Type::Label,
            Value::Grid(_) => Type::Grid,
            Value::Flex(_) => Type::Flex,
            Value::GridEntry(_) => Type::GridEntry,
            Value::Path(_) => Type::Path,
            Value::Image(_) => Type::Image,
            Value::ObjectFit(_) => Type::ObjectFit,
            Value::Dict(_) => Type::DynamicDict,
            Value::VerticalAlignment(_) => Type::VAlign,
            Value::HorizontalAlignment(_) => Type::HAlign,
            Value::TextAlign(_) => Type::TextAlign,
            Value::Font(_) => Type::Font,
            Value::StyleUnit(_) => Type::StyleUnit,
            Value::UserFunction(_) => todo!(),
            Value::Thickness(_) => Type::Thickness,
            Value::CustomElement(_) | Value::Array(_) => unreachable!("Not possible"),
            Value::Filter(_) => Type::Filter,
            Value::TextStyling(_) => Type::TextStyling,
            Value::Element(_) => Type::Element,
            Value::Animation(_) => Type::Animation,
            Value::Position(_) => Type::Position,
            Value::Module(_) => Type::Module(crate::ModuleIndex::ANY),
        }
    }

    pub fn parse_string_literal(
        text: &str,
        replace_escapisms: bool,
        includes_quotes: bool,
    ) -> Value {
        if text.contains('\n') {
            parse_multiline_string(text, replace_escapisms, includes_quotes)
        } else {
            parse_single_line_string(text, replace_escapisms, includes_quotes)
        }
    }

    #[track_caller]
    pub fn convert_to_element(self) -> slides_rs_core::Element {
        self.try_convert_to_element().expect("Valid element")
    }

    #[track_caller]
    pub fn try_convert_to_element(self) -> Option<slides_rs_core::Element> {
        Some(match self {
            Value::Label(element) => element.into(),
            Value::Grid(element) => element.into(),
            Value::Flex(element) => element.into(),
            Value::Image(element) => element.into(),
            Value::CustomElement(element) => element.into(),
            Value::Element(element) => element.into(),
            _ => return None,
        })
    }

    pub fn convert_to_string(self) -> String {
        match self {
            Value::None(_) => String::new(),
            Value::String(string) => string,
            Value::Float(float) => float.to_string(),
            Value::Integer(int) => int.to_string(),
            Value::StyleUnit(style_unit) => style_unit.to_string(),
            Value::Color(color) => color.to_string(),
            Value::Path(path) => path.to_string_lossy().to_string(),
            Value::Void(_) => unreachable!(),
            Value::StyleReference(styling_reference) => format!("{styling_reference}"),
            Value::Background(background) => todo!(),
            Value::Label(label) => todo!(),
            Value::Grid(grid) => todo!(),
            Value::Flex(flex) => todo!(),
            Value::GridEntry(grid_entry) => todo!(),
            Value::Image(image) => todo!(),
            Value::ObjectFit(object_fit) => todo!(),
            Value::VerticalAlignment(vertical_alignment) => todo!(),
            Value::HorizontalAlignment(horizontal_alignment) => todo!(),
            Value::TextAlign(text_align) => todo!(),
            Value::Font(font) => todo!(),
            Value::Dict(hash_map) => todo!(),
            Value::UserFunction(user_function_value) => todo!(),
            Value::CustomElement(custom_element) => todo!(),
            Value::Thickness(thickness) => todo!(),
            Value::Array(values) => todo!(),
            Value::Filter(filter) => todo!(),
            Value::Animation(animation) => todo!(),
            Value::TextStyling(rw_lock) => todo!(),
            Value::Element(element) => todo!(),
            Value::Position(position) => todo!(),
            Value::Module(module) => todo!(),
        }
    }

    pub fn as_mut_base_element(&self) -> slides_rs_core::ElementRefMut {
        match self {
            Value::Label(label) => slides_rs_core::ElementRefMut::Label(label.clone()),
            Value::Image(image) => slides_rs_core::ElementRefMut::Image(image.clone()),
            Value::CustomElement(custom_element) => {
                slides_rs_core::ElementRefMut::CustomElement(custom_element.clone())
            }
            Value::Grid(grid) => slides_rs_core::ElementRefMut::Grid(grid.clone()),
            Value::Flex(flex) => slides_rs_core::ElementRefMut::Flex(flex.clone()),
            _ => unreachable!("Self is not a base element!"),
        }
    }

    pub fn as_string_array(&self) -> Vec<String> {
        match self {
            Value::Array(values) => values.into_iter().map(|v| v.as_string().clone()).collect(),
            _ => unreachable!("Value is not a string array!"),
        }
    }

    pub fn as_element_array(&self) -> Vec<Element> {
        match self {
            Value::Array(values) => values
                .into_iter()
                .map(|v| v.clone().convert_to_element())
                .collect(),
            _ => unreachable!("Value is not a string array!"),
        }
    }
}

fn parse_multiline_string(text: &str, _replace_escapisms: bool, includes_quotes: bool) -> Value {
    let text = if includes_quotes {
        text.strip_suffix("\"\"\"")
            .expect("valid string literal")
            .strip_prefix("\"\"\"")
            .expect("valid string literal")
    } else {
        text
    };
    let mut result = String::with_capacity(text.len());
    let mut is_start = true;
    let mut indent = 0;
    for line in text.lines() {
        let line = if is_start && line.is_empty() {
            continue;
        } else if line.is_empty() {
            result.push('\n');
            continue;
        } else if !is_start {
            &line[indent.min(line.len())..]
        } else {
            line
        };
        let mut tmp = line.chars();
        while let Some(ch) = tmp.next() {
            match ch {
                ' ' if is_start => {
                    indent += 1;
                }
                _ => {
                    is_start = false;
                    result.push(ch);
                }
            }
        }
        result.push('\n');
    }
    // Remove trailing whitespace
    let trunc = result
        .as_bytes()
        .iter()
        .enumerate()
        .rev()
        .skip_while(|(_, b)| b.is_ascii_whitespace())
        .map(|(i, _)| i + 1)
        .next()
        .unwrap_or(result.len());
    result.truncate(trunc);
    Value::String(result)
}

fn parse_single_line_string(text: &str, _replace_escapisms: bool, includes_quotes: bool) -> Value {
    let text = if includes_quotes {
        text.strip_suffix('"')
            .expect("valid string literal")
            .strip_prefix('"')
            .expect("valid string literal")
    } else {
        text
    };
    let mut result = String::with_capacity(text.len());
    let mut tmp = text.chars();
    while let Some(ch) = tmp.next() {
        match ch {
            _ => result.push(ch),
        }
    }
    Value::String(result)
}

impl From<slides_rs_core::Label> for Value {
    fn from(value: slides_rs_core::Label) -> Self {
        Self::Label(Arc::new(RwLock::new(value)))
    }
}

impl From<slides_rs_core::Image> for Value {
    fn from(value: slides_rs_core::Image) -> Self {
        Self::Image(Arc::new(RwLock::new(value)))
    }
}

impl From<slides_rs_core::CustomElement> for Value {
    fn from(value: slides_rs_core::CustomElement) -> Self {
        Self::CustomElement(Arc::new(RwLock::new(value)))
    }
}

impl From<slides_rs_core::Grid> for Value {
    fn from(value: slides_rs_core::Grid) -> Self {
        Self::Grid(Arc::new(RwLock::new(value)))
    }
}

impl From<slides_rs_core::Flex> for Value {
    fn from(value: slides_rs_core::Flex) -> Self {
        Self::Flex(Arc::new(RwLock::new(value)))
    }
}
