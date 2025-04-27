use std::{collections::HashMap, sync::atomic::AtomicBool};

use bindings::{
    component::arrows::{
        slides,
        types::Type,
        values::{Value, ValueIndex},
    },
    exports::component::arrows::modules::{self, Guest, GuestModule, Module},
};

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    type Module = Arrows;
}

struct Arrows {
    is_library_downloaded: AtomicBool,
}
impl Arrows {
    fn arrow(
        &self,
        slides: slides::Slides,
        allocator: &mut modules::ValueAllocator,
        from: u32,
        to: u32,
        parent: Option<u32>,
        namespace: &str,
        options: HashMap<String, ValueIndex>,
    ) -> Result<(), modules::Error> {
        use std::fmt::Write;
        let is_library_downloaded = self
            .is_library_downloaded
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |_| Some(true),
            )
            .unwrap();
        if !is_library_downloaded {
            slides.download_file("https://raw.githubusercontent.com/wert007/leader-line/refs/heads/master/leader-line.min.js", "pros-assets/leader-line.min.js");
            slides.add_file_reference("pros-assets/leader-line.min.js");
            slides.place_text_in_output(
                "<script src=\"pros-assets/leader-line.min.js\"></script>",
                "arrows module",
                slides::Placement::HtmlHead,
            );
        }
        let mut text = String::new();
        let mut options_text = String::new();
        fn value_to_string(value: &Value) -> Result<String, modules::Error> {
            Ok(match value {
                Value::Int(num) => num.to_string(),
                Value::StringType(string) => format!("\"{string}\""),
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
                "color" | "path" | "startSocket" | "endSocket" => {
                    writeln!(options_text, "{key}: {},", value_to_string(&value)?)
                        .expect("infallible");
                }
                "start_socket" => {
                    writeln!(options_text, "startSocket: {},", value_to_string(&value)?)
                        .expect("Infallible");
                }
                "end_socket" => {
                    writeln!(options_text, "endSocket: {},", value_to_string(&value)?)
                        .expect("Infallible");
                }
                "middle_label" => {
                    writeln!(options_text, "middleLabel: {},", value_to_string(&value)?)
                        .expect("Infallible");
                }
                _ => {
                    return Err(modules::Error::InternalError(format!(
                        "Invalid option: {key}"
                    )));
                }
            }
        }
        let parent_option = match parent {
            Some(parent) => format!("getElementById({parent})"),
            None => format!("document.getElementById(\"{namespace}\")"),
        };
        writeln!(
            text,
            "
    new LeaderLine(
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
            is_library_downloaded: AtomicBool::new(false),
        })
    }

    fn available_functions(&self) -> Vec<modules::Function> {
        vec![modules::Function {
            name: "arrow".into(),
            args: vec![Type::Element, Type::Element, Type::Dict],
            result_type: bindings::component::arrows::types::Type::Void,
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
                let namespace = if namespace_to.len() < namespace_from.len() {
                    namespace_to
                } else {
                    namespace_from
                };
                let parent = allocator.get(args[1]).try_into_element()?.parent;
                let options = allocator.get(args[2]).try_into_dict()?;
                self.arrow(
                    slides,
                    &mut allocator,
                    from,
                    to,
                    parent,
                    &namespace,
                    options,
                )?;
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
