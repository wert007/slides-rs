use std::collections::HashMap;

use strum::IntoEnumIterator;

use crate::{
    Location, ModuleIndex, Modules, StringInterner,
    compiler::module::state::{HostTypeAllocator, HostTypeIndex},
};

use super::{ConversionKind, Variable, globals};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionType {
    pub min_argument_count: usize,
    pub(crate) argument_types: Vec<TypeId>,
    pub(crate) return_type: TypeId,
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
    pub const FLOAT: TypeId = TypeId(2);
    pub const INTEGER: TypeId = TypeId(3);
    pub const BOOL: TypeId = TypeId(4);
    pub const STRING: TypeId = TypeId(5);
    pub const DICT: TypeId = TypeId(6);
    pub const PATH: TypeId = TypeId(7);
    pub const STYLING: TypeId = TypeId(8);
    pub const BACKGROUND: TypeId = TypeId(9);
    pub const COLOR: TypeId = TypeId(10);
    pub const STYLE_UNIT: TypeId = TypeId(16);
    pub const ELEMENT: TypeId = TypeId(18);
    pub const ANIMATION: TypeId = TypeId(27);

    pub unsafe fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
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
        debug_assert_eq!(result.get_or_intern(Type::Float), TypeId::FLOAT);
        debug_assert_eq!(result.get_or_intern(Type::Integer), TypeId::INTEGER);
        debug_assert_eq!(result.get_or_intern(Type::Bool), TypeId::BOOL);
        debug_assert_eq!(result.get_or_intern(Type::String), TypeId::STRING);
        debug_assert_eq!(result.get_or_intern(Type::DynamicDict), TypeId::DICT);
        debug_assert_eq!(result.get_or_intern(Type::Path), TypeId::PATH);
        debug_assert_eq!(result.get_or_intern(Type::Styling), TypeId::STYLING);
        debug_assert_eq!(result.get_or_intern(Type::Background), TypeId::BACKGROUND);
        debug_assert_eq!(result.get_or_intern(Type::Color), TypeId::COLOR);
        debug_assert_eq!(result.get_or_intern(Type::StyleUnit), TypeId::STYLE_UNIT);
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

    pub fn add_from_module(
        &mut self,
        type_allocator: &mut HostTypeAllocator,
        string_interner: &mut StringInterner,
    ) {
        let mut new_types_mapping = HashMap::with_capacity(type_allocator.types.len());
        for (index, type_) in &type_allocator.types {
            self.convert_module_type(
                *index,
                type_,
                &type_allocator,
                string_interner,
                &mut new_types_mapping,
            );
        }
        type_allocator.types = new_types_mapping;
    }

    fn convert_module_type(
        &mut self,
        index: HostTypeIndex,
        type_: &crate::compiler::module::component::arrows::types::Type,
        type_allocator: &HostTypeAllocator,
        string_interner: &mut StringInterner,
        new_types_mapping: &mut HashMap<
            HostTypeIndex,
            crate::compiler::module::component::arrows::types::Type,
        >,
    ) -> TypeId {
        let type_id = match type_ {
            crate::compiler::module::component::arrows::types::Type::Void => {
                self.get_or_intern(Type::Void)
            }
            crate::compiler::module::component::arrows::types::Type::String => {
                self.get_or_intern(Type::String)
            }
            crate::compiler::module::component::arrows::types::Type::Int => {
                self.get_or_intern(Type::Integer)
            }
            crate::compiler::module::component::arrows::types::Type::Float => {
                self.get_or_intern(Type::Float)
            }
            crate::compiler::module::component::arrows::types::Type::Element => {
                self.get_or_intern(Type::Element)
            }
            crate::compiler::module::component::arrows::types::Type::Dict => {
                self.get_or_intern(Type::DynamicDict)
            }
            crate::compiler::module::component::arrows::types::Type::Enum(name) => {
                self.get_or_intern(Type::Enum(name.clone()))
            }
            crate::compiler::module::component::arrows::types::Type::EnumDefinition((
                enum_type_index,
                variants,
            )) => {
                let enum_type = type_allocator.get(*enum_type_index);
                let enum_type = self.convert_module_type(
                    enum_type_index.clone().into(),
                    enum_type,
                    type_allocator,
                    string_interner,
                    new_types_mapping,
                );
                let enum_type = self.resolve(enum_type).clone();
                self.get_or_intern(Type::EnumDefinition(Box::new(enum_type), variants.clone()))
            }
            crate::compiler::module::component::arrows::types::Type::Color => {
                self.get_or_intern(Type::Color)
            }
            crate::compiler::module::component::arrows::types::Type::Bool => {
                self.get_or_intern(Type::Bool)
            }
            crate::compiler::module::component::arrows::types::Type::Struct((name, fields)) => {
                let mut entries = Vec::with_capacity(fields.len());
                for (name, type_index) in fields {
                    let type_ = type_allocator.get(*type_index);
                    let type_ = self.convert_module_type(
                        type_index.clone().into(),
                        type_,
                        type_allocator,
                        string_interner,
                        new_types_mapping,
                    );
                    let name = string_interner.create_or_get_variable(name);
                    entries.push(Variable {
                        id: name,
                        definition: Location::zero(),
                        type_,
                    });
                }
                self.get_or_intern(Type::TypedDict(entries))
            }
            crate::compiler::module::component::arrows::types::Type::Array(type_index) => {
                let type_ = type_allocator.get(*type_index);
                let inner = self.convert_module_type(
                    type_index.clone().into(),
                    type_,
                    type_allocator,
                    string_interner,
                    new_types_mapping,
                );
                self.get_or_intern(Type::Array(inner))
            }
            crate::compiler::module::component::arrows::types::Type::Optional(type_index) => {
                let type_ = type_allocator.get(*type_index);
                let inner = self.convert_module_type(
                    type_index.clone().into(),
                    type_,
                    type_allocator,
                    string_interner,
                    new_types_mapping,
                );
                self.get_or_intern(Type::Optional(inner))
            }
        };
        let index = HostTypeIndex {
            relocateable: type_id.0,
            fixed: index.fixed,
        };
        new_types_mapping.insert(index, type_.clone());
        type_id
    }

    pub fn id_to_simple_string(&self, id: TypeId) -> String {
        let type_ = self.resolve(id);
        self.to_simple_string(type_)
    }

    pub(crate) fn to_simple_string(&self, t: &Type) -> String {
        match t {
            Type::Error => unreachable!(),
            Type::Void => "void".into(),
            Type::Float => "float".into(),
            Type::Integer => "int".into(),
            Type::Bool => "bool".into(),
            Type::String => "string".into(),
            Type::TypedDict(_) | Type::DynamicDict => "dict".into(),
            Type::Path => "Path".into(),
            Type::Styling => "Style".into(),
            Type::Background => "Background".into(),
            Type::Color => "Color".into(),
            Type::ObjectFit => "ObjectFit".into(),
            Type::HAlign => "HAlign".into(),
            Type::VAlign => "VAlign".into(),
            Type::TextAlign => "TextAlign".into(),
            Type::Font => "Font".into(),
            Type::StyleUnit => "StyleUnit".into(),
            Type::Function(_function_type) => "function".into(),
            Type::Slide => "Slide".into(),
            Type::Element => "Element".into(),
            Type::Label => "Label".into(),
            Type::Grid => "Grid".into(),
            Type::Flex => "Flex".into(),
            Type::GridEntry => "GridEntry".into(),
            Type::Image => "Image".into(),
            Type::Thickness => "Thickness".into(),
            Type::Enum(name) => name.clone(),
            Type::EnumDefinition(base, _) => self.to_simple_string(base),
            Type::CustomElement(name, _) => name.clone(),
            Type::Array(type_id) => {
                let name = self.id_to_simple_string(*type_id);
                format!("{name}[]")
            }
            Type::Optional(type_id) => {
                let name = self.id_to_simple_string(*type_id);
                format!("{name}?")
            }
            Type::Filter => "Filter".into(),
            Type::TextStyling => "TextStyling".into(),
            Type::Animation => "Animation".into(),
            Type::Position => "Position".into(),
            Type::Module(_) => "module".into(),
        }
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
    Enum(String),
    EnumDefinition(Box<Type>, Vec<String>),
    CustomElement(String, HashMap<String, TypeId>),
    Array(TypeId),
    Optional(TypeId),
    Filter,
    TextStyling,
    Animation,
    Position,
    Module(ModuleIndex),
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

    pub fn field_type(
        &self,
        member: &str,
        type_interner: &mut TypeInterner,
        modules: &Modules,
    ) -> Option<Type> {
        if self == &Type::Error {
            return Some(Type::Error);
        }
        if let Type::EnumDefinition(result, variants) = self {
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
        if let Type::Module(index) = self {
            let module = &modules[*index];
            return module
                .read()
                .unwrap()
                .try_get_function_by_name(member)
                .map(|f| Type::Function(f.type_.clone()));
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
                    Type::EnumDefinition(..)
                        | Type::Enum(_)
                        | Type::Function(_)
                        | Type::CustomElement(..)
                        | Type::Array(_)
                        | Type::Optional(_)
                        | Type::TypedDict(_)
                        | Type::Module(_)
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
