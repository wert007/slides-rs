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
        from: u32,
        to: u32,
        parent: u32,
        options: HashMap<String, ValueIndex>,
    ) {
        use std::fmt::Write;
        let is_library_downloaded = self
            .is_library_downloaded
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |x| Some(x),
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
        assert!(options.is_empty());
        writeln!(
            text,
            "
    new LeaderLine(
        getElementById({from}),
        getElementById({to}),
        {{
            parent: getElementById({parent}),
        }},
    );"
        )
        .expect("infallible");
        slides.place_text_in_output(
            &text,
            "arrows arrow function",
            slides::Placement::JavascriptInit,
        );
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
        allocator: modules::ValueAllocator,
        args: Vec<modules::ValueIndex>,
    ) -> Result<modules::ValueIndex, modules::Error> {
        Ok(match name.as_str() {
            "arrow" => {
                if args.len() != 3 {
                    return Err(modules::Error::ArgumentCountMismatch);
                }
                let from = allocator.get(args[0]).try_into_element()?.id;
                let to = allocator.get(args[1]).try_into_element()?.id;
                let Some(parent) = allocator.get(args[1]).try_into_element()?.parent else {
                    return Err(modules::Error::InternalError("parent must be set".into()));
                };
                let options = allocator.get(args[2]).try_into_dict()?;
                self.arrow(slides, from, to, parent, options);
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
