use std::collections::HashMap;

use strum::IntoEnumIterator;

use super::{ConversionKind, Variable, globals};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionType {
    pub min_argument_count: usize,
    pub(super) argument_types: Vec<TypeId>,
    pub(super) return_type: TypeId,
}
impl FunctionType {
    pub fn return_type(&self) -> TypeId {
        self.return_type
    }

    pub fn argument_count(&self) -> usize {
        self.argument_types.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypeId(usize);

impl TypeId {
    pub const ERROR: TypeId = TypeId(0);
    pub const VOID: TypeId = TypeId(1);
    pub const INTEGER: TypeId = TypeId(3);
    pub const BOOL: TypeId = TypeId(4);
    pub const STRING: TypeId = TypeId(5);
    pub const DICT: TypeId = TypeId(6);
    pub const PATH: TypeId = TypeId(7);
    pub const STYLING: TypeId = TypeId(8);
    pub const BACKGROUND: TypeId = TypeId(9);
    pub const COLOR: TypeId = TypeId(10);
    pub const ELEMENT: TypeId = TypeId(18);
    pub const ANIMATION: TypeId = TypeId(27);
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
        debug_assert_eq!(result.get_or_intern(Type::Integer), TypeId::INTEGER);
        debug_assert_eq!(result.get_or_intern(Type::Bool), TypeId::BOOL);
        debug_assert_eq!(result.get_or_intern(Type::String), TypeId::STRING);
        debug_assert_eq!(result.get_or_intern(Type::DynamicDict), TypeId::DICT);
        debug_assert_eq!(result.get_or_intern(Type::Path), TypeId::PATH);
        debug_assert_eq!(result.get_or_intern(Type::Styling), TypeId::STYLING);
        debug_assert_eq!(result.get_or_intern(Type::Background), TypeId::BACKGROUND);
        debug_assert_eq!(result.get_or_intern(Type::Color), TypeId::COLOR);
        debug_assert_eq!(result.get_or_intern(Type::Element), TypeId::ELEMENT);
        debug_assert_eq!(result.get_or_intern(Type::Animation), TypeId::ANIMATION);
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

    pub fn resolve(&self, id: TypeId) -> &Type {
        self.types.get(id.0).expect("TypeIds are always valid")
    }

    pub fn resolve_types<const N: usize>(&self, target: [TypeId; N]) -> [&Type; N] {
        target.map(|t| self.types.get(t.0).expect("TypeIds are always valid"))
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
    Bool,
    String,
    DynamicDict,
    Path,
    TypedDict(Vec<Variable>),
    Styling,
    Background,
    Color,
    ObjectFit,
    HAlign,
    VAlign,
    TextAlign,
    Font,
    StyleUnit,
    Function(FunctionType),
    Slide,
    Element,
    Label,
    Grid,
    Flex,
    GridEntry,
    Image,
    Thickness,
    Enum(Box<Type>, Vec<String>),
    CustomElement(String, HashMap<String, TypeId>),
    Array(TypeId),
    Filter,
    TextStyling,
    Animation,
    Position,
}

impl Type {
    pub fn get_available_conversions(&self, kind: ConversionKind) -> &'static [Type] {
        match kind {
            ConversionKind::Implicit => match self {
                Type::Integer => &[Type::Float],
                Type::Color => &[Type::Background],
                Type::Label | Type::Image | Type::CustomElement(_, _) => &[Type::Element],
                _ => &[],
            },
            ConversionKind::TypedString => match self {
                Type::String => &[Type::Color, Type::Label, Type::Path],
                _ => &[],
            },
            ConversionKind::ToString => match self {
                Type::Float | Type::Integer | Type::String | Type::Path => &[Type::String],
                _ => &[],
            },
        }
    }

    pub fn field_type(&self, member: &str, type_interner: &mut TypeInterner) -> Option<Type> {
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
        if let Type::CustomElement(_, members) = self {
            if let Some(type_) = members.get(member) {
                return Some(type_interner.resolve(*type_).clone());
            }
        }
        for m in globals::MEMBERS {
            if self.as_ref() != m.name {
                continue;
            }
            let Some(index) = m.members_names.iter().position(|n| n == &member) else {
                continue;
            };
            let rs_type_name = globals::normalize_type_name(m.members_rust_types[index].trim());
            return Some(
                Self::from_rust_string(rs_type_name)
                    .or_else(|| Self::from_fn_name(rs_type_name, type_interner))
                    .unwrap_or_else(|| panic!("Could not find type! {m:?}.{member}")),
            );
        }
        None
    }

    pub const fn from_rust_string(rust_string: &str) -> Option<Self> {
        if let Some((desc, type_)) = konst::string::split_once(rust_string, ':') {
            let Some(type_) = Self::from_rust_string_primitive_id(type_) else {
                return None;
            };

            if konst::eq_str(desc, "Array") {
                Some(Self::Array(type_))
            } else {
                panic!("Invalid descriptor");
            }
        } else if let Some((desc, type_)) = konst::string::split_once(rust_string, '<') {
            let type_ = konst::string::strip_suffix(type_, '>')
                .expect("Leading < should be followed by >.");
            let Some(type_) = Self::from_rust_string_primitive_id(type_) else {
                return None;
            };

            if konst::eq_str(desc, "Vec") {
                Some(Self::Array(type_))
            } else {
                panic!("Invalid descriptor");
            }
        } else {
            Self::from_rust_string_primitive(rust_string)
        }
    }

    pub fn simple_types() -> Vec<Type> {
        Type::iter()
            .filter(|t| {
                !matches!(
                    t,
                    Type::Enum(..)
                        | Type::Function(_)
                        | Type::CustomElement(_, _)
                        | Type::Array(_)
                        | Type::TypedDict(_)
                )
            })
            .collect()
    }

    fn from_fn_name(name: &str, type_interner: &mut TypeInterner) -> Option<Type> {
        let name = name.strip_prefix("#Fn(")?;
        let (parameters, return_type) = name.split_once("):")?;
        let parameters: Option<Vec<TypeId>> = parameters
            .split(',')
            .into_iter()
            .map(|p| Type::from_rust_string(p).map(|t| type_interner.get_or_intern(t)))
            .collect();
        let parameters = parameters?;
        let return_type = type_interner.get_or_intern(Self::from_rust_string(return_type)?);
        Some(Type::Function(FunctionType {
            min_argument_count: parameters.len(),
            argument_types: parameters,
            return_type,
        }))
    }

    const fn from_rust_string_primitive_id(rust_string: &str) -> Option<TypeId> {
        if konst::eq_str(rust_string, "Element") {
            Some(TypeId::ELEMENT)
        } else if konst::eq_str(rust_string, "StyleReference") {
            Some(TypeId::STYLING)
        } else if konst::eq_str(rust_string, "Animation") {
            Some(TypeId::ANIMATION)
        } else {
            None
        }
    }

    const fn from_rust_string_primitive(rust_string: &str) -> Option<Type> {
        if konst::eq_str(rust_string, "()") {
            Some(Self::Void)
        } else if konst::eq_str(rust_string, "f64") {
            Some(Self::Float)
        } else if konst::eq_str(rust_string, "bool") {
            Some(Self::Bool)
        } else if konst::eq_str(rust_string, "i64") {
            Some(Self::Integer)
        } else if konst::eq_str(rust_string, "String") {
            Some(Self::String)
        } else if konst::eq_str(rust_string, "Background") {
            Some(Self::Background)
        } else if konst::eq_str(rust_string, "Color") {
            Some(Self::Color)
        } else if konst::eq_str(rust_string, "Label") {
            Some(Self::Label)
        } else if konst::eq_str(rust_string, "Grid") {
            Some(Self::Grid)
        } else if konst::eq_str(rust_string, "Image") {
            Some(Self::Image)
        } else if konst::eq_str(rust_string, "ObjectFit") {
            Some(Self::ObjectFit)
        } else if konst::eq_str(rust_string, "PathBuf") {
            Some(Self::Path)
        } else if konst::eq_str(rust_string, "TextAlign") {
            Some(Self::TextAlign)
        } else if konst::eq_str(rust_string, "Font") {
            Some(Self::Font)
        } else if konst::eq_str(rust_string, "HorizontalAlignment") {
            Some(Self::HAlign)
        } else if konst::eq_str(rust_string, "VerticalAlignment") {
            Some(Self::VAlign)
        } else if konst::eq_str(rust_string, "Thickness") {
            Some(Self::Thickness)
        } else if konst::eq_str(rust_string, "StringArray") {
            Some(Self::Array(TypeId::STRING))
        } else if konst::eq_str(rust_string, "StyleUnit") {
            Some(Self::StyleUnit)
        } else if konst::eq_str(rust_string, "usize") {
            Some(Self::Integer)
        } else if konst::eq_str(rust_string, "Filter") {
            Some(Self::Filter)
        } else if konst::eq_str(rust_string, "Animation") {
            Some(Self::Animation)
        } else if konst::eq_str(rust_string, "TextStyling") {
            Some(Self::TextStyling)
        } else if konst::eq_str(rust_string, "GridEntry") {
            Some(Self::GridEntry)
        } else if konst::eq_str(rust_string, "Element") {
            Some(Self::Element)
        } else if konst::eq_str(rust_string, "Flex") {
            Some(Self::Flex)
        } else if konst::eq_str(rust_string, "Position") {
            Some(Self::Position)
        } else {
            None
        }
    }
}
