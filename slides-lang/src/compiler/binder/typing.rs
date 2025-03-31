use strum::IntoEnumIterator;

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
    pub fn field_type(&self, member: &str) -> Option<Type> {
        match self {
            Type::Error => Some(Type::Error),
            Type::Label => match member {
                "text_color" => Some(Type::Color),
                "background" => Some(Type::Background),
                "align_center" => Some(Type::Function(FunctionType {
                    argument_types: Vec::new(),
                    return_type: TypeId::VOID,
                })),
                _ => None,
            },
            Type::Image => match member {
                "background" => Some(Type::Background),
                "object_fit" => Some(Type::ObjectFit),
                "halign" => Some(Type::HAlign),
                "valign" => Some(Type::VAlign),
                _ => None,
            },
            Type::Enum(result, variants) => {
                if variants.iter().any(|v| v == member) {
                    Some(*result.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
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
        } else if const_str::compare!(==, rust_string, "PathBuf") {
            Some(Self::Path)
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
