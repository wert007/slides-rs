use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicU32},
};

use bindings::{
    component::arrows::{
        slides,
        types::Type,
        values::{Value, ValueIndex},
    },
    exports::component::arrows::modules::{self, Guest, GuestModule, Module},
};

use crate::bindings::component::arrows::types::TypeIndex;

const JS_LIBRARY: &'static str = include_str!("connector.js");

// #[allow(warnings)]
// mod bindings;

mod bindings {
    wit_bindgen::generate!({
        // the name of the world in the `*.wit` input file
        world: "host",
    });
}

struct Component;

impl Guest for Component {
    type Module = Arrows;
}

struct Arrows {
    is_library_initiated: AtomicBool,
    line_options_key: AtomicU32,
}
impl Arrows {
    fn arrow(
        &self,
        slides: slides::Slides,
        allocator: &mut modules::ValueAllocator,
        from: u32,
        to: u32,
        namespace: &str,
        options: HashMap<String, ValueIndex>,
    ) -> Result<(), modules::Error> {
        use std::fmt::Write;
        let is_library_initiated = self
            .is_library_initiated
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |_| Some(true),
            )
            .unwrap();
        if !is_library_initiated {
            slides.place_text_in_output(
                &format!("<script>{JS_LIBRARY}</script>"),
                "arrows module",
                slides::Placement::HtmlHead,
            );

            slides.place_text_in_output(
                "
            globals.arrows = new SimpleConnector()",
                "module arrows import",
                slides::Placement::JavascriptInit,
            );

            slides.place_text_in_output(
                "globals.arrows.updateAll();",
                "module arrows import",
                slides::Placement::JavascriptSlideChange,
            );
        }
        let mut text = String::new();
        let mut options_text = String::new();
        fn value_to_string(
            value: &Value,
            allocator: &modules::ValueAllocator,
        ) -> Result<String, modules::Error> {
            Ok(match value {
                Value::Int(num) => num.to_string(),
                Value::Float(num) => num.to_string(),
                Value::StringType(string) => format!("\"{string}\""),
                Value::Dict(dict) => {
                    let mut result: String = "{".into();
                    for (key, value) in dict {
                        result.push_str(key);
                        result.push(':');
                        result.push_str(&value_to_string(&allocator.get(*value), allocator)?);
                        result.push(',');
                    }
                    result.push('}');
                    result
                }
                _ => {
                    return Err(modules::Error::InternalError(format!(
                        "Value is not supported in options: {value:#?}"
                    )));
                }
            })
        }
        for (key, value) in options {
            let value = allocator.get(value);
            match key.as_str() {
                "color" | "width" | "from_pos" | "to_pos" | "line_kind" | "label" => {
                    writeln!(
                        options_text,
                        "{key}: {},",
                        value_to_string(&value, allocator)?
                    )
                    .expect("infallible");
                }
                "middle_label" => {
                    writeln!(
                        options_text,
                        "middleLabel: LeaderLine.captionLabel({}, {{ classList: 'label'}}),",
                        value_to_string(&value, allocator)?
                    )
                    .expect("Infallible");
                }
                _ => {
                    return Err(modules::Error::InternalError(format!(
                        "Invalid option: {key}"
                    )));
                }
            }
        }
        let parent_option = format!("document.getElementById(\"{namespace}\")");
        writeln!(
            text,
            "
    globals.arrows.connect(
        getElementById({from}),
        getElementById({to}),
        {{
            parent: {parent_option},
            {options_text}
        }},
    );"
        )
        .expect("infallible");
        slides.place_text_in_output(
            &text,
            "arrows arrow function",
            slides::Placement::JavascriptInit,
        );
        Ok(())
    }
}

impl GuestModule for Arrows {
    fn create(_slides: slides::Slides) -> modules::Module {
        Module::new(Self {
            is_library_initiated: AtomicBool::new(false),
            line_options_key: AtomicU32::default(),
        })
    }

    fn register_types(&self, types: modules::TypeAllocator) -> () {
        let line_tip_kind = types.allocate(&Type::Enum("LineTip".into()));
        types.allocate(&Type::EnumDefinition((
            line_tip_kind,
            ["Arrow", "Triangle", "Square", "Circle", "None"]
                .into_iter()
                .map(Into::into)
                .collect(),
        )));
        let boolean = types.allocate(&Type::Bool);
        let line_tip = types.allocate(&Type::Struct((
            "TipOptions".into(),
            [
                ("kind".into(), line_tip_kind),
                ("filled".into(), boolean),
                ("flip".into(), boolean),
            ]
            .to_vec(),
        )));

        let line_kind = types.allocate(&Type::Enum("LineKind".into()));
        types.allocate(&Type::EnumDefinition((
            line_kind,
            ["Direct", "Orthogonal"]
                .into_iter()
                .map(Into::into)
                .collect(),
        )));
        let float = types.allocate(&Type::Float);
        let point = types.allocate(&Type::Struct((
            "Point".into(),
            [("x".into(), float), ("y".into(), float)].to_vec(),
        )));
        let color = types.allocate(&Type::Color);
        let create_optional = |base: TypeIndex| types.allocate(&Type::Optional(base));
        let line_options = types.allocate(&Type::Struct((
            "LineOptions".into(),
            [
                ("width".into(), create_optional(float)),
                ("color".into(), create_optional(color)),
                ("kind".into(), create_optional(line_kind)),
                ("starttip".into(), create_optional(line_tip)),
                ("endtip".into(), create_optional(line_tip)),
                ("relative_pos_start".into(), create_optional(point)),
                ("relative_pos_end".into(), create_optional(point)),
            ]
            .to_vec(),
        )));
        self.line_options_key.store(
            line_options.fixed_unique_key,
            std::sync::atomic::Ordering::SeqCst,
        );
    }

    fn available_functions(&self, types: modules::TypeAllocator) -> Vec<modules::Function> {
        let element = types.allocate(&Type::Element);
        let void = types.allocate(&Type::Void);
        let options = types.get_by_key(
            self.line_options_key
                .load(std::sync::atomic::Ordering::SeqCst),
        );
        vec![modules::Function {
            name: "arrow".into(),
            args: vec![element, element, options],
            result_type: void,
        }]
    }

    fn call_function(
        &self,
        slides: slides::Slides,
        name: String,
        mut allocator: modules::ValueAllocator,
        args: Vec<modules::ValueIndex>,
    ) -> Result<modules::ValueIndex, modules::Error> {
        Ok(match name.as_str() {
            "arrow" => {
                if args.len() != 3 {
                    return Err(modules::Error::ArgumentCountMismatch);
                }
                let from = allocator.get(args[0]).try_into_element()?.id;
                let namespace_from = allocator.get(args[0]).try_into_element()?.namespace;
                let to = allocator.get(args[1]).try_into_element()?.id;
                let namespace_to = allocator.get(args[1]).try_into_element()?.namespace;
                let namespace = namespace_from
                    .split_once('-')
                    .map(|(n, _)| n)
                    .unwrap_or(&namespace_from);
                if !namespace_to.starts_with(namespace) {
                    return Err(modules::Error::InternalError(format!(
                        "Elements have different namespaces: {namespace_from} and {namespace_to}"
                    )));
                }
                let options = allocator.get(args[2]).try_into_dict()?;
                self.arrow(slides, &mut allocator, from, to, &namespace, options)?;
                allocator.allocate(&Value::Void)
                // allocator.
            }
            _ => return Err(modules::Error::FunctionNotFound),
        })
    }
}

impl Value {
    pub(crate) fn try_into_dict(&self) -> Result<HashMap<String, ValueIndex>, modules::Error> {
        match self {
            Value::Dict(dict) => Ok(dict.iter().cloned().collect()),
            _ => Err(modules::Error::InvalidType),
        }
    }
    pub(crate) fn try_into_element(
        &self,
    ) -> Result<bindings::component::arrows::values::Element, modules::Error> {
        match self {
            Value::Element(element) => Ok(element.clone()),
            _ => Err(modules::Error::InvalidType),
        }
    }
}

bindings::export!(Component with_types_in bindings);
