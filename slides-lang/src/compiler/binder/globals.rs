use slides_rs_core::{HorizontalAlignment, ObjectFit, SlidesEnum, TextAlign, VerticalAlignment};

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
