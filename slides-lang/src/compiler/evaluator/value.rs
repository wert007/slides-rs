use std::{cell::RefCell, collections::HashMap, path::PathBuf, sync::Arc};

use summum_types::summum;

use crate::{
    VariableId,
    compiler::binder::{
        BoundNode,
        typing::{Type, TypeId},
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

summum! {
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    pub enum Value {
        Void(()),
        Float(f64),
        Integer(i64),
        String(String),
        StyleReference(slides_rs_core::StylingReference),
        Background(slides_rs_core::Background),
        Color(slides_rs_core::Color),
        Label(Arc<RefCell<slides_rs_core::Label>>),
        Grid(Arc<RefCell<slides_rs_core::Grid>>),
        Path(PathBuf),
        Image(Arc<RefCell<slides_rs_core::Image>>),
        ObjectFit(slides_rs_core::ObjectFit),
        VerticalAlignment(slides_rs_core::VerticalAlignment),
        HorizontalAlignment(slides_rs_core::HorizontalAlignment),
        TextAlign(slides_rs_core::TextAlign),
        Font(slides_rs_core::Font),
        StyleUnit(slides_rs_core::StyleUnit),
        Dict(HashMap<String, Value>),
        UserFunction(UserFunctionValue),
        CustomElement(Arc<RefCell<slides_rs_core::CustomElement>>),
        Thickness(slides_rs_core::Thickness),
        Array(Vec<Value>),
        Filter(slides_rs_core::Filter),
        TextStyling(Arc<RefCell<slides_rs_core::TextStyling>>),
    }
}

impl Value {
    pub fn infer_type(&self) -> Type {
        match self {
            Value::Void(()) => Type::Void,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
            Value::StyleReference(_) => Type::Styling,
            Value::Background(_) => Type::Background,
            Value::Color(_) => Type::Color,
            Value::Label(_) => Type::Label,
            Value::Grid(_) => Type::Grid,
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
            Value::CustomElement(e) => Type::CustomElement(e.borrow().type_name().into()),
            Value::Thickness(_) => Type::Thickness,
            Value::Array(_) => unreachable!("Not possible"),
            Value::Filter(_) => Type::Filter,
            Value::TextStyling(_) => Type::TextStyling,
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

    pub fn as_mut_base_element(&self) -> slides_rs_core::ElementRefMut {
        match self {
            Value::Label(label) => slides_rs_core::ElementRefMut::Label(label.clone()),
            Value::Image(image) => slides_rs_core::ElementRefMut::Image(image.clone()),
            Value::CustomElement(custom_element) => {
                slides_rs_core::ElementRefMut::CustomElement(custom_element.clone())
            }
            _ => unreachable!("Self is not a base element!"),
        }
    }

    pub fn as_string_array(&self) -> Vec<String> {
        match self {
            Value::Array(values) => values.into_iter().map(|v| v.as_string().clone()).collect(),
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
        Self::Label(Arc::new(RefCell::new(value)))
    }
}

impl From<slides_rs_core::Image> for Value {
    fn from(value: slides_rs_core::Image) -> Self {
        Self::Image(Arc::new(RefCell::new(value)))
    }
}

impl From<slides_rs_core::CustomElement> for Value {
    fn from(value: slides_rs_core::CustomElement) -> Self {
        Self::CustomElement(Arc::new(RefCell::new(value)))
    }
}

impl From<slides_rs_core::Grid> for Value {
    fn from(value: slides_rs_core::Grid) -> Self {
        Self::Grid(Arc::new(RefCell::new(value)))
    }
}
