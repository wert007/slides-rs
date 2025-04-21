use convert_case::{Case, Casing};
use enum_dispatch::enum_dispatch;
use struct_field_names_as_array::FieldNamesAsSlice;

use crate::{GridEntry, HorizontalAlignment, Result, StyleUnit, Thickness, VerticalAlignment};
use std::{any::type_name, fmt::Display, io::Write, ops::Deref};

#[enum_dispatch]
pub trait ToCss {
    fn class_name(&self) -> String;

    fn to_css_rule(
        &self,
        layout: ToCssLayout,
        selector: &str,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        let style = self.to_css_style(layout);
        if style.is_empty() {
            Ok(())
        } else {
            writeln!(w, "{selector} {{\n{style}}}\n")
        }
    }

    fn to_css_style(&self, layout: ToCssLayout) -> String;

    fn collect_google_font_references(
        &self,
        fonts: &mut std::collections::HashSet<String>,
    ) -> Result<()>;
}

impl ToCss for () {
    fn class_name(&self) -> String {
        String::new()
    }

    fn to_css_style(&self, _layout: ToCssLayout) -> String {
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

#[derive(Debug, strum::Display)]
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
            unexpected => unreachable!("Expected Label, found {unexpected}"),
        }
    }
    pub fn as_image_mut(&mut self) -> &mut ImageStyling {
        match &mut self.specific {
            Styling::Image(image_styling) => image_styling,
            unexpected => unreachable!("Expected Image, found {unexpected}"),
        }
    }
    pub fn as_slide_mut(&mut self) -> &mut SlideStyling {
        match &mut self.specific {
            Styling::Slide(slide_styling) => slide_styling,
            unexpected => unreachable!("Expected Slide, found {unexpected}"),
        }
    }
}

impl ToCss for DynamicElementStyling {
    fn to_css_rule(
        &self,
        layout: ToCssLayout,
        selector: &str,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        self.base.to_css_rule(layout.clone(), selector, w)?;
        self.specific.to_css_rule(layout.clone(), selector, w)?;
        Ok(())
    }
    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        unreachable!("PANIC");
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Filter {
    #[default]
    Unspecified,
    Brightness(f64),
}

impl Filter {
    pub fn to_css(&self) -> String {
        match self {
            Filter::Unspecified => "unset".into(),
            Filter::Brightness(b) => format!("brightness({b})"),
        }
    }
}

#[derive(Debug, Default, Clone, struct_field_names_as_array::FieldNamesAsSlice)]
pub struct BaseElementStyling {
    pub background: Background,
    pub halign: HorizontalAlignment,
    pub valign: VerticalAlignment,
    pub margin: Thickness,
    pub padding: Thickness,
    pub filter: Filter,
    pub width: StyleUnit,
    pub height: StyleUnit,
    pub is_visible: bool,
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

    pub fn set_width(&mut self, width: StyleUnit) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: StyleUnit) {
        self.height = height;
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }
}

impl ToCss for BaseElementStyling {
    fn to_css_style(&self, layout: ToCssLayout) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut translate = (0.0, 0.0);

        let mut was_height_already_emitted = false;
        let mut was_width_already_emitted = false;
        match self.valign {
            VerticalAlignment::Unset => {}
            VerticalAlignment::Top => {
                writeln!(
                    result,
                    "    top: {};\n    bottom: unset;",
                    self.margin.top.max(layout.outer_padding.top).or_zero()
                )
                .expect("infallible");
            }
            VerticalAlignment::Center => {
                writeln!(result, "    top: 50%;\n    bottom: unset;").expect("infallible");
                translate.1 = -50.0;
            }
            VerticalAlignment::Bottom => {
                writeln!(
                    result,
                    "    bottom: {};\n    top: unset;",
                    self.margin
                        .bottom
                        .max(layout.outer_padding.bottom)
                        .or_zero()
                )
                .expect("infallible");
            }
            VerticalAlignment::Stretch => {
                let top = self.margin.top.max(layout.outer_padding.top);
                let bottom = self.margin.bottom.max(layout.outer_padding.bottom);
                let height = StyleUnit::Percent(100.0) - top - bottom;
                let top = top.or_zero();
                let bottom = bottom.or_zero();
                was_height_already_emitted = true;
                writeln!(
                    result,
                    "    top: {top};\n    bottom: {bottom};\n    height: {height};",
                )
                .expect("infallible");
            }
        }

        match self.halign {
            HorizontalAlignment::Unset => {}
            HorizontalAlignment::Left => {
                writeln!(
                    result,
                    "    left: {};\n    right: unset;",
                    self.margin.left.max(layout.outer_padding.left).or_zero()
                )
                .expect("infallible");
            }
            HorizontalAlignment::Center => {
                writeln!(result, "    left: 50%;\n    right: unset;").expect("infallible");
                translate.0 = -50.0;
            }
            HorizontalAlignment::Right => {
                writeln!(
                    result,
                    "    right: {};\n    left: unset;",
                    self.margin.right.max(layout.outer_padding.right).or_zero()
                )
                .expect("infallible");
            }
            HorizontalAlignment::Stretch => {
                let left = self.margin.left.max(layout.outer_padding.left);
                let right = self.margin.right.max(layout.outer_padding.right);
                let width = StyleUnit::Percent(100.0) - left - right;
                let left = left.or_zero();
                let right = right.or_zero();
                was_width_already_emitted = true;
                writeln!(
                    result,
                    "    left: {left};\n    right: {right};\n    width: {width};",
                )
                .expect("infallible");
            }
        }

        if !was_height_already_emitted && self.height != StyleUnit::Unspecified {
            writeln!(result, "    height: {};", self.height).expect("infallible");
        }

        if !was_width_already_emitted && self.width != StyleUnit::Unspecified {
            writeln!(result, "    width: {};", self.width).expect("infallible");
        }

        if translate != (0.0, 0.0) {
            writeln!(
                result,
                "    transform: translate({}%, {}%);",
                translate.0, translate.1
            )
            .expect("infallible");
        }

        if self.padding != Thickness::default() {
            writeln!(result, "    padding: {};", self.padding).expect("infallible");
        }

        if let Some(z_index) = self.z_index {
            writeln!(result, "    z-index: {z_index};").expect("infallible");
        }

        if self.background != Background::Unspecified {
            writeln!(result, "    background: {};", self.background).expect("infallible");
        }

        if self.filter != Filter::Unspecified {
            writeln!(result, "    filter: {};", self.filter.to_css()).expect("infallible");
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

    pub fn base(&self) -> &BaseElementStyling {
        &self.base
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

#[derive(Debug, Clone, Copy)]
pub struct ToCssLayout {
    pub outer_padding: Thickness,
    pub grid_data: Option<GridEntry>,
}
impl ToCssLayout {
    pub(crate) fn unknown() -> ToCssLayout {
        Self {
            outer_padding: Thickness::default(),
            grid_data: None,
            // animation_init_values: Vec::new(),
        }
    }

    // pub(crate) fn new(base: &BaseElementStyling) -> Self {
    //     Self {
    //         outer_padding: base.padding,
    //         grid_data: None,
    //     }
    // }
}

impl<S: ToCss> ToCss for ElementStyling<S> {
    fn to_css_rule(
        &self,
        layout: ToCssLayout,
        selector: &str,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        if let Some(grid_data) = layout.grid_data {
            writeln!(w, "{selector} {{")?;
            if grid_data.column_span != 1 {
                writeln!(w, "    grid-column: span {};", grid_data.column_span)?;
            }
            if grid_data.row_span != 1 {
                writeln!(w, "    grid-row: span {};", grid_data.row_span)?;
            }
            writeln!(w, "}}\n")?;
        }
        self.base.to_css_rule(layout.clone(), selector, w)?;
        self.specific.to_css_rule(layout.clone(), selector, w)?;
        Ok(())
    }
    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        unreachable!("PANIC");
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

    pub fn set_text_color(&mut self, color: Color) {
        self.specific.text_color = Some(color);
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
pub struct SlideStyling {
    text_color: Option<Color>,
}

impl SlideStyling {
    pub fn new() -> ElementStyling<SlideStyling> {
        ElementStyling::new(Self { text_color: None })
    }
}

impl ToCss for SlideStyling {
    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if let Some(text_color) = self.text_color {
            writeln!(result, "    color: {text_color};").expect("infallible");
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
        "slide".into()
    }
}

#[derive(Default, Debug, PartialEq, Eq, strum::Display, Clone)]
pub enum Font {
    #[strum(to_string = "unset")]
    #[default]
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

#[derive(Default, Debug, Clone, FieldNamesAsSlice, PartialEq)]
pub struct TextStyling {
    text_color: Option<Color>,
    text_align: TextAlign,
    font: Font,
    font_size: StyleUnit,
}

impl TextStyling {
    pub fn set_text_color(&mut self, text_color: Color) {
        self.text_color = Some(text_color);
    }

    fn output_css_statements(&self, w: &mut dyn Write) -> std::io::Result<()> {
        if let Some(text_color) = self.text_color {
            writeln!(w, "    color: {text_color};")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, FieldNamesAsSlice, Default, PartialEq)]
pub struct LabelStyling {
    text: TextStyling,
    text_color: Option<Color>,
    text_align: TextAlign,
    font: Font,
    font_size: Option<f64>,
}

impl LabelStyling {
    pub fn new() -> ElementStyling<LabelStyling> {
        ElementStyling::new(Self {
            text: TextStyling::default(),
            text_color: None,
            text_align: TextAlign::Unspecified,
            font: Font::Unspecified,
            font_size: None,
        })
    }

    pub fn set_text_styling(&mut self, text: TextStyling) {
        self.text = text;
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

    pub fn set_font_size(&mut self, font_size: f64) {
        self.font_size = Some(font_size);
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

    pub fn set_font_size(&mut self, font_size: f64) {
        self.specific.font_size = Some(font_size);
    }
}

impl ToCss for LabelStyling {
    fn to_css_rule(
        &self,
        _layout: ToCssLayout,
        selector: &str,
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        if self == &LabelStyling::default() {
            return Ok(());
        }
        if self.text != TextStyling::default() {
            writeln!(w, "{selector} .label-text {{")?;
            self.text.output_css_statements(w)?;
            writeln!(w, "}}\n")?;
            writeln!(w, "{selector} {{")?;
        } else {
            writeln!(w, "{selector}, {selector} .label-text {{")?;
        }
        if let Some(text_color) = self.text_color {
            writeln!(w, "    color: {text_color};").expect("infallible");
        }
        if self.font != Font::Unspecified {
            writeln!(w, "    font-family: {};", self.font).expect("infallible");
        }
        if self.text_align != TextAlign::Unspecified {
            writeln!(w, "    text-align: {};", self.text_align.as_css()).expect("infallible");
        }
        if let Some(font_size) = self.font_size {
            writeln!(
                w,
                "    font-size: calc({font_size} * min(16 * 4dvh, 9 * 4dvw) / 16);"
            )
            .expect("infallible");
        }
        writeln!(w, "}}\n")?;

        Ok(())
    }

    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        unreachable!("PANIC!");
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
    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if self.object_fit != ObjectFit::Unspecified {
            writeln!(result, "    object-fit: {};", self.object_fit.as_css()).expect("infallible");
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

#[derive(Debug, Clone)]
pub enum GridCellSize {
    Auto,
    Fraction(usize),
    Concrete(StyleUnit),
    Minimum,
    Maximum,
}

impl GridCellSize {
    pub fn to_css(&self) -> String {
        match self {
            GridCellSize::Auto => "auto".into(),
            GridCellSize::Fraction(fr) => format!("{fr}fr"),
            GridCellSize::Concrete(style_unit) => style_unit.to_string(),
            GridCellSize::Minimum => "min-content".into(),
            GridCellSize::Maximum => "max-content".into(),
        }
    }
}

#[derive(Debug, Clone, FieldNamesAsSlice)]
pub struct GridStyling {
    columns: Vec<GridCellSize>,
    rows: Vec<GridCellSize>,
}

impl GridStyling {
    pub fn new(columns: Vec<GridCellSize>, rows: Vec<GridCellSize>) -> ElementStyling<Self> {
        ElementStyling::new(GridStyling { columns, rows })
    }
}

impl ToCss for GridStyling {
    fn class_name(&self) -> String {
        "grid".into()
    }

    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if !self.columns.is_empty() {
            writeln!(
                result,
                "    grid-template-columns: {};",
                self.columns
                    .iter()
                    .map(|c| c.to_css())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .expect("Infallibe");
        }
        if !self.rows.is_empty() {
            writeln!(
                result,
                "    grid-template-rows: {};",
                self.rows
                    .iter()
                    .map(|c| c.to_css())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .expect("Infallibe");
        }
        result
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Unspecified,
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    pub fn to_css(&self) -> String {
        match self {
            FlexDirection::Unspecified => "unset".into(),
            FlexDirection::Row => "row".into(),
            FlexDirection::Column => "column".into(),
            FlexDirection::RowReverse => "row-reverse".into(),
            FlexDirection::ColumnReverse => "column-reverse".into(),
        }
    }
}

#[derive(Debug, Clone, FieldNamesAsSlice)]
pub struct FlexStyling {
    direction: FlexDirection,
}

impl FlexStyling {
    pub fn new() -> ElementStyling<Self> {
        ElementStyling::new(FlexStyling {
            direction: FlexDirection::Unspecified,
        })
    }
}

impl ToCss for FlexStyling {
    fn class_name(&self) -> String {
        "flex".into()
    }

    fn to_css_style(&self, _layout: ToCssLayout) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        if self.direction != FlexDirection::Unspecified {
            writeln!(result, "    flex-direction: {};", self.direction.to_css())
                .expect("infallibe");
        }
        result
    }

    fn collect_google_font_references(
        &self,
        _: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        Ok(())
    }
}

impl ElementStyling<FlexStyling> {
    pub fn set_direction(&mut self, direction: FlexDirection) {
        self.specific.direction = direction;
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
