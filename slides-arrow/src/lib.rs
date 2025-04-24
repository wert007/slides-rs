use bindings::{
    component::arrows::values::Value,
    exports::component::arrows::modules::{self, Guest, GuestModule, Module},
};

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    type Module = Arrows;
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
        allocator: modules::ValueAllocator,
        args: Vec<modules::ValueIndex>,
    ) -> Result<modules::ValueIndex, modules::Error> {
        Ok(match name.as_str() {
            "arrow" => allocator.allocate(&Value::Void),
            _ => return Err(modules::Error::FunctionNotFound),
        })
    }
}

bindings::export!(Component with_types_in bindings);
