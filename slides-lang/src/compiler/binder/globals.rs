#![allow(unused_mut)]

use constcat::concat_slices;
use slides_rs_core::{
    BaseElementStyling, HorizontalAlignment, ImageStyling, LabelStyling, ObjectFit, SlideStyling,
    SlidesEnum, TextAlign, TextStyling, VerticalAlignment,
};

include!(concat!(env!("OUT_DIR"), "/global_functions.rs"));

#[derive(Debug)]
pub struct EnumDeclaration {
    pub name: &'static str,
    pub type_: Type,
    pub variants: &'static [&'static str],
}

impl EnumDeclaration {
    const fn rename<T: SlidesEnum>(name: &'static str, type_: Type) -> EnumDeclaration {
        EnumDeclaration {
            name,
            type_,
            variants: T::VARIANTS,
        }
    }
}

pub const ENUMS: [EnumDeclaration; 4] = [
    EnumDeclaration::rename::<ObjectFit>("ObjectFit", Type::ObjectFit),
    EnumDeclaration::rename::<HorizontalAlignment>("HAlign", Type::HAlign),
    EnumDeclaration::rename::<VerticalAlignment>("VAlign", Type::VAlign),
    EnumDeclaration::rename::<TextAlign>("TextAlign", Type::TextAlign),
];

#[derive(Debug)]
pub struct MemberDeclarations {
    pub name: &'static str,
    pub members_names: &'static [&'static str],
    pub members_rust_types: &'static [&'static str],
}
impl MemberDeclarations {
    const fn rename<T: struct_field_names_as_array::FieldNamesAsSlice>(
        name: &'static str,
    ) -> MemberDeclarations {
        Self {
            name,
            members_names: T::FIELD_NAMES_AS_SLICE,
            members_rust_types: T::FIELD_TYPES_AS_SLICE,
        }
    }

    // const fn chain(self, other: Self) -> Self {
    //     Self { name: self.name, members_names: concat_slices!([&'static str]: self.members_names, other.members_names), members_rust_types: () }
    // }
}

macro_rules! chain_member_decls {
    ($a:expr, $b:expr) => {
        MemberDeclarations { name: $a.name, members_names: concat_slices!([&'static str]: $a.members_names, $b.members_names), members_rust_types: concat_slices!([&'static str]: $a.members_rust_types, $b.members_rust_types) }

    };
}

macro_rules! default_element {
    ($name:literal) => {
        chain_member_decls!(
            MemberDeclarations::rename::<BaseElementStyling>($name),
            MemberDeclarations {
                name: $name,
                members_names: &["styles"],
                members_rust_types: &["#ArrayOfStyleReferences",],
            }
        )
    };
}

pub const MEMBERS: [MemberDeclarations; 8] = [
    MemberDeclarations::rename::<TextStyling>("TextStyling"),
    MemberDeclarations::rename::<LabelStyling>("Label"),
    default_element!("Label"),
    MemberDeclarations::rename::<ImageStyling>("Image"),
    default_element!("Image"),
    default_element!("Element"),
    MemberDeclarations::rename::<SlideStyling>("Slide"),
    default_element!("Slide"),
];

pub(crate) fn normalize_type_name(mut name: &str) -> &str {
    if let Some(rs_type_name_without_option) = name
        .strip_prefix("Option <")
        .and_then(|t| t.strip_suffix('>'))
    {
        name = rs_type_name_without_option.trim();
    }
    name
}

pub(crate) fn find_members_by_name(
    name: &str,
) -> impl Iterator<Item = (&'static str, Type)> + 'static {
    let name = name.to_owned();
    MEMBERS
        .iter()
        .filter(move |m| m.name == name)
        .flat_map(|m| {
            m.members_names
                .iter()
                .copied()
                .zip(m.members_rust_types.iter().map(|r| {
                    Type::from_rust_string(normalize_type_name(r))
                        .unwrap_or_else(|| panic!("{r} should exist"))
                }))
        })
}
