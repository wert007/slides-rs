use bindings::exports::component::arrows::modules::{self, Guest, GuestModule, Module};

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    type Module = Arrows;

    fn hello() -> String {
        "Hello".into()
    }
}

struct Arrows;

impl GuestModule for Arrows {
    fn create() -> modules::Module {
        Module::new(Self)
    }

    fn available_functions(&self) -> Vec<modules::Function> {
        vec![modules::Function {
            name: "arrow".into(),
            args: vec![],
            result_type: bindings::component::arrows::types::Type::Void,
        }]
    }

    fn call_function(
        &self,
        name: String,
        args: Vec<modules::Value>,
    ) -> Result<modules::Value, modules::Error> {
        Ok(match name.as_str() {
            "arrow" => modules::Value::Void,
            _ => return Err(modules::Error::FunctionNotFound),
        })
    }
}

bindings::export!(Component with_types_in bindings);
