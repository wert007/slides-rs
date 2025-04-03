use std::collections::HashMap;

use slides_rs_core::{
    BaseElementStyling, HorizontalAlignment, ImageStyling, LabelStyling, ObjectFit, SlideStyling,
    SlidesEnum, TextAlign, VerticalAlignment,
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
}

pub const MEMBERS: [MemberDeclarations; 5] = [
    MemberDeclarations::rename::<LabelStyling>("Label"),
    MemberDeclarations::rename::<ImageStyling>("Image"),
    MemberDeclarations::rename::<BaseElementStyling>("Element"),
    MemberDeclarations::rename::<SlideStyling>("Slide"),
    MemberDeclarations {
        name: "Element",
        members_names: &["valign", "halign"],
        members_rust_types: &["VerticalAlignment", "HorizontalAlignment"],
    },
];
