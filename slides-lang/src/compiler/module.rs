use std::{
    io::{Read, Write},
    path::PathBuf,
};

use component::arrows;
use exports::component::arrows::modules;
use wasmtime::{
    Config, Store,
    component::{Component, Linker, bindgen},
};

use crate::{Context, Location, VariableId, compiler::binder::typing::FunctionType};

use super::binder::{Binder, Variable, typing};

// mod wai {
//     use crate::compiler::binder::Binder as TypeChecker;
//     use crate::compiler::evaluator::Evaluator;
//     wai_bindgen_rust::import!("../arrows-module/module.wai");
// }

#[derive(Debug, Clone)]
pub struct Module {
    pub name: VariableId,
    functions: Vec<(String, FunctionType)>,
}

impl Module {
    pub fn try_get_function_by_name(&self, name: &str) -> Option<&FunctionType> {
        self.functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
    }

    pub(crate) fn try_call_function_by_name(
        &self,
        name: String,
    ) -> Option<super::evaluator::value::Value> {
        todo!()
    }
}

bindgen!({
    path: "../slides-arrow/wit/world.wit",
    // with: {
    //     "wasi": wasmtime_wasi::bindings,
    // }
});

struct State {}

impl arrows::types::Host for State {}
impl arrows::values::Host for State {}

pub fn load_module(
    name: VariableId,
    path: impl Into<PathBuf>,
    binder: &mut Binder,
    context: &mut Context,
) -> std::io::Result<Module> {
    let path = path.into();
    dbg!(path.display());
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    dbg!(archive.file_names().collect::<Vec<_>>());
    let mut file = archive.by_name("slides_arrow.wasm")?;
    let mut buffer = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buffer)?;

    // let contents = std::fs::read(&path).unwrap();
    // assert!(contents.starts_with(b"\0asm"));
    let engine = wasmtime::Engine::default();

    let component = Component::from_binary(&engine, &buffer).unwrap();

    let mut linker = Linker::new(&engine);
    linker.allow_shadowing(true);
    linker.define_unknown_imports_as_traps(&component).unwrap();
    Host_::add_to_linker(&mut linker, |state: &mut State| state).unwrap();

    let mut store = Store::new(&engine, State { /* ... */ });
    let bindings = Host_::instantiate(&mut store, &component, &linker).unwrap();

    let module = bindings
        .component_arrows_modules()
        .module()
        .call_create(&mut store)
        .unwrap();
    let functions = bindings
        .component_arrows_modules()
        .module()
        .call_available_functions(&mut store, module)
        .unwrap();

    Ok(Module {
        name,
        functions: functions
            .into_iter()
            .map(|f| {
                let type_ = FunctionType {
                    min_argument_count: f.args.len(),
                    argument_types: f
                        .args
                        .into_iter()
                        .map(|t| context.type_interner.get_or_intern(t.into()))
                        .collect(),
                    return_type: context.type_interner.get_or_intern(f.result_type.into()),
                };
                (f.name, type_)
            })
            .collect(),
    })
}

impl From<modules::Type> for typing::Type {
    fn from(value: modules::Type) -> Self {
        match value {
            arrows::types::Type::Void => Self::Void,
            arrows::types::Type::String => Self::String,
            arrows::types::Type::Int => Self::Integer,
            arrows::types::Type::Float => Self::Float,
        }
    }
}
