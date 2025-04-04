use convert_case::{Case, Casing};
use enum_dispatch::enum_dispatch;
use struct_field_names_as_array::FieldNamesAsSlice;

use crate::{HorizontalAlignment, Result, StyleUnit, Thickness, VerticalAlignment};
use std::{any::type_name, fmt::Display, ops::Deref};

#[enum_dispatch]
pub trait ToCss {
    fn class_name(&self) -> String;

    fn to_css_style(&self) -> String;

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()>;
}

impl ToCss for () {
    fn class_name(&self) -> String {
        String::new()
    }

    fn to_css_style(&self) -> String {
        String::new()
    }

    fn collect_google_font_references(
        &self,
        _fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StylingReference {
    name: String,
}

impl StylingReference {
    pub unsafe fn from_raw(name: String) -> StylingReference {
        Self { name }
    }
}

impl Display for StylingReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
#[enum_dispatch(ToCss)]
pub enum Styling {
    Label(LabelStyling),
    Image(ImageStyling),
    Slide(SlideStyling),
    CustomElement(()),
}

#[derive(Debug)]
pub struct DynamicElementStyling {
    name: String,
    base: BaseElementStyling,
    specific: Styling,
}

impl DynamicElementStyling {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn as_base_mut(&mut self) -> &mut BaseElementStyling {
        &mut self.base
    }

    pub fn as_label_mut(&mut self) -> &mut LabelStyling {
        match &mut self.specific {
            Styling::Label(label_styling) => label_styling,
            _ => unreachable!("Expected Label"),
        }
    }
    pub fn as_image_mut(&mut self) -> &mut ImageStyling {
        match &mut self.specific {
            Styling::Image(image_styling) => image_styling,
            _ => unreachable!("Expected Image"),
        }
    }
    pub fn as_slide_mut(&mut self) -> &mut SlideStyling {
        match &mut self.specific {
            Styling::Slide(slide_styling) => slide_styling,
            _ => unreachable!("Expected Slide"),
        }
    }
}

impl ToCss for DynamicElementStyling {
    fn to_css_style(&self) -> String {
        [self.base.to_css_style(), self.specific.to_css_style()].join("\n")
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        self.specific.collect_google_font_references(fonts)
    }

    fn class_name(&self) -> String {
        self.specific.class_name()
    }
}

#[derive(Debug, Default, Clone, struct_field_names_as_array::FieldNamesAsSlice)]
pub struct BaseElementStyling {
    background: Background,
    halign: HorizontalAlignment,
    valign: VerticalAlignment,
    margin: Thickness,
    padding: Thickness,
    z_index: Option<usize>,
}
impl BaseElementStyling {
    pub fn set_background(&mut self, background: Background) {
        self.background = background;
    }

    pub fn set_horizontal_alignment(&mut self, horizontal_alignment: HorizontalAlignment) {
        self.halign = horizontal_alignment;
    }

    pub fn set_vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) {
        self.valign = vertical_alignment;
    }

    pub fn set_z_index(&mut self, z_index: usize) {
        self.z_index = Some(z_index);
    }

    pub fn set_margin(&mut self, margin: Thickness) {
        self.margin = margin;
    }

    pub fn set_padding(&mut self, padding: Thickness) {
        self.padding = padding;
    }
}

impl ToCss for BaseElementStyling {
    fn to_css_style(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut translate = (0.0, 0.0);

        match self.valign {
            VerticalAlignment::Unset => {}
            VerticalAlignment::Top => {
                writeln!(result, "top: {}; bottom: unset;", self.margin.top.or_zero())
                    .expect("infallible");
            }
            VerticalAlignment::Center => {
                writeln!(result, "top: 50%; bottom: unset;").expect("infallible");
                translate.1 = -50.0;
            }
            VerticalAlignment::Bottom => {
                writeln!(
                    result,
                    "bottom: {}; top: unset;",
                    self.margin.bottom.or_zero()
                )
                .expect("infallible");
            }
            VerticalAlignment::Stretch => {
                writeln!(
                    result,
                    "top: {};\nbottom: {};\nheight: 100%;",
                    self.margin.top.or_zero(),
                    self.margin.bottom.or_zero()
                )
                .expect("infallible");
            }
        }

        match self.halign {
            HorizontalAlignment::Unset => {}
            HorizontalAlignment::Left => {
                writeln!(
                    result,
                    "left: {}; right: unset;",
                    self.margin.left.or_zero()
                )
                .expect("infallible");
            }
            HorizontalAlignment::Center => {
                writeln!(result, "left: 50%; right: unset;").expect("infallible");
                translate.0 = -50.0;
            }
            HorizontalAlignment::Right => {
                writeln!(
                    result,
                    "right: {}; left: unset;",
                    self.margin.right.or_zero()
                )
                .expect("infallible");
            }
            HorizontalAlignment::Stretch => {
                writeln!(
                    result,
                    "left: {};\right: {};\nwidth: 100%;",
                    self.margin.left.or_zero(),
                    self.margin.right.or_zero()
                )
                .expect("infallible");
            }
        }

        if translate != (0.0, 0.0) {
            writeln!(
                result,
                "transform: translate({}%, {}%);",
                translate.0, translate.1
            )
            .expect("infallible");
        }

        if self.padding != Thickness::default() {
            writeln!(result, "padding: {};", self.padding).expect("infallible");
        }

        if let Some(z_index) = self.z_index {
            writeln!(result, "z-index: {z_index};").expect("infallible");
        }

        if self.background != Background::Unspecified {
            writeln!(result, "background: {};", self.background).expect("infallible");
        }
        result
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn class_name(&self) -> String {
        unreachable!("This can only be called on element stylings!")
    }
}

#[derive(Debug, Clone)]
pub struct ElementStyling<S> {
    base: BaseElementStyling,
    specific: S,
}

impl<S> ElementStyling<S> {
    pub fn base_mut(&mut self) -> &mut BaseElementStyling {
        &mut self.base
    }
}

impl ElementStyling<()> {
    pub fn new_base() -> Self {
        Self {
            base: BaseElementStyling::default(),
            specific: (),
        }
    }
}

impl<S> Deref for ElementStyling<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.specific
    }
}

impl<S: ToCss + 'static> ElementStyling<S> {
    pub fn with_background(mut self, background: Background) -> Self {
        self.base.background = background;
        self
    }

    fn new(specific: S) -> Self {
        Self {
            base: BaseElementStyling::default(),
            specific,
        }
    }

    pub fn set_background(&mut self, background: Background) {
        self.base.background = background;
    }
}

impl<S: ToCss> ToCss for ElementStyling<S> {
    fn to_css_style(&self) -> String {
        [self.base.to_css_style(), self.specific.to_css_style()].join("\n")
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        self.specific.collect_google_font_references(fonts)
    }

    fn class_name(&self) -> String {
        self.specific.class_name()
    }
}

impl ElementStyling<LabelStyling> {
    pub fn to_dynamic(self, name: String) -> DynamicElementStyling {
        let name = if name == "default" {
            self.class_name()
        } else {
            name
        };
        DynamicElementStyling {
            name,
            base: self.base,
            specific: self.specific.into(),
        }
    }
}

impl ElementStyling<ImageStyling> {
    pub fn to_dynamic(self, name: String) -> DynamicElementStyling {
        let name = if name == "default" {
            self.class_name()
        } else {
            name
        };
        DynamicElementStyling {
            name,
            base: self.base,
            specific: self.specific.into(),
        }
    }
}
impl ElementStyling<SlideStyling> {
    pub fn to_dynamic(self, name: String) -> DynamicElementStyling {
        let name = if name == "default" {
            self.class_name()
        } else {
            name
        };
        DynamicElementStyling {
            name,
            base: self.base,
            specific: self.specific.into(),
        }
    }
}
impl ElementStyling<()> {
    pub fn to_dynamic(self, name: String) -> DynamicElementStyling {
        let name = if name == "default" {
            self.class_name()
        } else {
            name
        };
        DynamicElementStyling {
            name,
            base: self.base,
            specific: self.specific.into(),
        }
    }
}

#[derive(Debug, Default, FieldNamesAsSlice)]
pub struct SlideStyling {}

impl SlideStyling {
    pub fn new() -> ElementStyling<SlideStyling> {
        ElementStyling::new(Self {})
    }
}

impl ToCss for SlideStyling {
    fn to_css_style(&self) -> String {
        String::new()
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn class_name(&self) -> String {
        "slide".into()
    }
}

#[derive(Debug, PartialEq, Eq, strum::Display, Clone)]
pub enum Font {
    #[strum(to_string = "unset")]
    Unspecified,
    #[strum(to_string = "\"{0}\"")]
    GoogleFont(String),
    #[strum(to_string = "\"{0}\"")]
    System(String),
}

impl Font {
    pub fn gfont(name: impl Into<String>) -> Self {
        Self::GoogleFont(name.into())
    }

    pub fn system(name: impl Into<String>) -> Self {
        Self::System(name.into())
    }
}

#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    strum::Display,
    Clone,
    Copy,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
// #[strum(serialize_all = "kebab-case")]
pub enum TextAlign {
    #[default]
    #[strum(to_string = "unset")]
    Unspecified,
    Left,
    Right,
    Center,
    Justify,
}

impl SlidesEnum for TextAlign {}

impl TextAlign {
    pub fn as_css(&self) -> String {
        self.to_string().to_case(Case::Kebab)
    }
}

#[derive(Debug, Clone, FieldNamesAsSlice)]
pub struct LabelStyling {
    text_color: Option<Color>,
    text_align: TextAlign,
    font: Font,
    font_size: StyleUnit,
}

impl LabelStyling {
    pub fn new() -> ElementStyling<LabelStyling> {
        ElementStyling::new(Self {
            text_color: None,
            text_align: TextAlign::Unspecified,
            font: Font::Unspecified,
            font_size: StyleUnit::Unspecified,
        })
    }
}

impl LabelStyling {
    pub fn set_text_color(&mut self, text_color: Color) {
        self.text_color = Some(text_color);
    }

    pub fn set_text_align(&mut self, text_align: TextAlign) {
        self.text_align = text_align;
    }

    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    pub fn set_font_size(&mut self, font_size: StyleUnit) {
        self.font_size = font_size;
    }
}

impl ElementStyling<LabelStyling> {
    pub fn with_text_color(mut self, text_color: Color) -> Self {
        self.specific.text_color = Some(text_color);
        self
    }

    pub fn with_font(mut self, font: Font) -> Self {
        self.specific.font = font;
        self
    }

    pub fn set_text_color(&mut self, text_color: Color) {
        self.specific.text_color = Some(text_color);
    }

    pub fn set_text_align(&mut self, text_align: TextAlign) {
        self.specific.text_align = text_align;
    }

    pub fn set_font_size(&mut self, font_size: StyleUnit) {
        self.specific.font_size = font_size;
    }
}

impl ToCss for LabelStyling {
    fn to_css_style(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if let Some(text_color) = self.text_color {
            writeln!(result, "color: {text_color};").expect("infallible");
        }
        if self.font != Font::Unspecified {
            writeln!(result, "font-family: {};", self.font).expect("infallible");
        }
        if self.text_align != TextAlign::Unspecified {
            writeln!(result, "text-align: {};", self.text_align.as_css()).expect("infallible");
        }
        if self.font_size != StyleUnit::Unspecified {
            writeln!(result, "font-size: {};", self.font_size).expect("infallible");
        }
        result
    }

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        match &self.font {
            Font::GoogleFont(name) => {
                fonts.insert(name.clone());
            }
            _ => {}
        }
        Ok(())
    }

    fn class_name(&self) -> String {
        "label".into()
    }
}

#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    strum::Display,
    Clone,
    Copy,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
// #[strum(serialize_all = "kebab-case")]
pub enum ObjectFit {
    #[default]
    #[strum(to_string = "unset")]
    Unspecified,
    Contain,
    None,
    Cover,
    Fill,
    ScaleDown,
}

impl ObjectFit {
    pub fn as_css(&self) -> String {
        self.to_string().to_case(Case::Kebab)
    }
}

pub trait SlidesEnum: ::strum::VariantNames {
    fn name() -> &'static str {
        type_name::<Self>()
    }
    fn variants() -> &'static [&'static str] {
        Self::VARIANTS
    }
}

impl SlidesEnum for ObjectFit {}

#[derive(Debug, Default, Clone, FieldNamesAsSlice)]
pub struct ImageStyling {
    object_fit: ObjectFit,
}

impl ImageStyling {
    pub fn new() -> ElementStyling<ImageStyling> {
        ElementStyling::new(ImageStyling::default())
    }
}

impl ElementStyling<ImageStyling> {
    pub fn with_object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.specific.object_fit = object_fit;
        self
    }

    pub fn set_object_fit(&mut self, object_fit: ObjectFit) {
        self.specific.object_fit = object_fit;
    }
}

impl ToCss for ImageStyling {
    fn to_css_style(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if self.object_fit != ObjectFit::Unspecified {
            writeln!(result, "object-fit: {};", self.object_fit.as_css()).expect("infallible");
        }
        result
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn class_name(&self) -> String {
        "image".into()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Background {
    #[default]
    Unspecified,
    Color(Color),
}

impl Display for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Background::Unspecified => write!(f, "unset"),
            Background::Color(color) => write!(f, "{color}"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
}

impl Color {
    pub const WHITE: Color = Self::rgb(0xff, 0xff, 0xff);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            alpha: 0xff,
        }
    }

    pub const fn argb(r: u8, g: u8, b: u8, alpha: u8) -> Self {
        Self { r, g, b, alpha }
    }

    pub fn from_css(color: &str) -> Self {
        csscolorparser::parse(color)
            .map(|color| {
                let [r, g, b, alpha] = color.to_rgba8();
                Color { r, g, b, alpha }
            })
            .unwrap_or_default()
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgb({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            self.alpha as f64 / 255.0
        )
    }
}
