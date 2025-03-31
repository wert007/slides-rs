use strum::IntoEnumIterator;

use super::{ConversionKind, globals};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionType {
    pub(super) argument_types: Vec<TypeId>,
    pub(super) return_type: TypeId,
}
impl FunctionType {
    pub fn return_type(&self) -> TypeId {
        self.return_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypeId(usize);

impl TypeId {
    pub const ERROR: TypeId = TypeId(0);
    pub const VOID: TypeId = TypeId(1);
    pub const DICT: TypeId = TypeId(5);
}

pub struct TypeInterner {
    types: Vec<Type>,
}

impl TypeInterner {
    pub fn new() -> Self {
        let types = Type::simple_types();
        let mut result = Self { types };
        debug_assert_eq!(result.get_or_intern(Type::Error), TypeId::ERROR);
        debug_assert_eq!(result.get_or_intern(Type::Void), TypeId::VOID);
        debug_assert_eq!(result.get_or_intern(Type::Dict), TypeId::DICT);
        result
    }

    pub fn get_or_intern(&mut self, type_: Type) -> TypeId {
        let index = match self.types.iter().position(|t| t == &type_) {
            Some(it) => it,
            None => {
                self.types.push(type_);
                self.types.len() - 1
            }
        };
        TypeId(index)
    }

    pub fn resolve(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id.0)
    }

    pub fn resolve_types<const N: usize>(&self, target: [TypeId; N]) -> [&Type; N] {
        target.map(|t| self.types.get(t.0).unwrap_or(&Type::Error))
    }
}

#[derive(
    Debug, strum::EnumTryAs, Clone, PartialEq, Eq, strum::EnumIter, Default, strum::AsRefStr,
)]
pub enum Type {
    #[default]
    Error,
    Void,
    Float,
    Integer,
    String,
    Dict,
    Styling,
    Background,
    Color,
    ObjectFit,
    HAlign,
    VAlign,
    TextAlign,
    Function(FunctionType),
    Slide,
    Element,
    Label,
    Image,
    Path,
    Enum(Box<Type>, Vec<String>),
    CustomElement(String),
}

impl Type {
    pub fn get_available_conversions(&self, kind: ConversionKind) -> &'static [Type] {
        match kind {
            ConversionKind::Implicit => match self {
                Type::Integer => &[Type::Float],
                Type::Color => &[Type::Background],
                Type::Label | Type::Image | Type::CustomElement(_) => &[Type::Element],
                _ => &[],
            },
            ConversionKind::TypedString => match self {
                Type::String => &[Type::Color, Type::Label, Type::Path],
                _ => &[],
            },
        }
    }

    pub fn field_type(&self, member: &str) -> Option<Type> {
        if self == &Type::Error {
            return Some(Type::Error);
        }
        if let Type::Enum(result, variants) = self {
            return if variants.iter().any(|v| v == member) {
                Some(*result.clone())
            } else {
                None
            };
        }
        for m in globals::MEMBERS {
            if self.as_ref() != m.name {
                continue;
            }
            let Some(index) = m.members_names.iter().position(|n| n == &member) else {
                continue;
            };
            let mut rs_type_name = m.members_rust_types[index].trim();
            if let Some(rs_type_name_without_option) = rs_type_name
                .strip_prefix("Option <")
                .and_then(|t| t.strip_suffix('>'))
            {
                rs_type_name = rs_type_name_without_option.trim();
            }
            return Some(
                Self::from_rust_string(rs_type_name)
                    .unwrap_or_else(|| panic!("Could not find type! {m:?}.{member}")),
            );
        }
        None
    }

    pub const fn from_rust_string(rust_string: &str) -> Option<Self> {
        if const_str::compare!(==, rust_string, "()" ) {
            Some(Self::Void)
        } else if const_str::compare!(==, rust_string, "f64") {
            Some(Self::Float)
        } else if const_str::compare!(==, rust_string, "i64") {
            Some(Self::Integer)
        } else if const_str::compare!(==, rust_string, "String") {
            Some(Self::String)
        } else if const_str::compare!(==, rust_string, "Background") {
            Some(Self::Background)
        } else if const_str::compare!(==, rust_string, "Color") {
            Some(Self::Color)
        } else if const_str::compare!(==, rust_string, "Label") {
            Some(Self::Label)
        } else if const_str::compare!(==, rust_string, "Image") {
            Some(Self::Image)
        } else if const_str::compare!(==, rust_string, "ObjectFit") {
            Some(Self::ObjectFit)
        } else if const_str::compare!(==, rust_string, "PathBuf") {
            Some(Self::Path)
        } else if const_str::compare!(==, rust_string, "TextAlign") {
            Some(Self::TextAlign)
        } else if const_str::compare!(==, rust_string, "HorizontalAlignment") {
            Some(Self::HAlign)
        } else if const_str::compare!(==, rust_string, "VerticalAlignment") {
            Some(Self::VAlign)
        } else {
            None
        }
    }

    pub fn simple_types() -> Vec<Type> {
        Type::iter()
            .filter(|t| {
                !matches!(
                    t,
                    Type::Enum(..) | Type::Function(_) | Type::CustomElement(_)
                )
            })
            .collect()
    }
}
