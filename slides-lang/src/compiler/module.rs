use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};

use component::arrows::{
    self,
    values::{self},
};
use exports::component::arrows::modules;
use slides_rs_core::Position;
use wasmtime::{
    Config, Store,
    component::{Component, Linker, Resource, ResourceAny, bindgen},
};

use crate::{Context, Location, VariableId, compiler::binder::typing::FunctionType};

use super::{
    binder::{Binder, Variable, typing},
    evaluator::value,
};

// mod wai {
//     use crate::compiler::binder::Binder as TypeChecker;
//     use crate::compiler::evaluator::Evaluator;
//     wai_bindgen_rust::import!("../arrows-module/module.wai");
// }

#[derive(Debug, Clone)]
pub struct ModuleFunction {
    pub name: String,
    pub type_: FunctionType,
}

pub struct Module {
    pub name: VariableId,
    functions: HashMap<String, ModuleFunction>,
    engine: wasmtime::Engine,
    store: wasmtime::Store<State>,
    bindings: Host_,
    this: ResourceAny,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name)
            .field("functions", &self.functions)
            .finish()
    }
}

impl Module {
    pub fn try_get_function_by_name(&self, name: &str) -> Option<&ModuleFunction> {
        self.functions
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| t)
    }

    pub fn try_call_function_by_name(
        &mut self,
        name: &str,
        arguments: Vec<value::Value>,
    ) -> Result<value::Value, modules::Error> {
        let module = self.bindings.component_arrows_modules().module();
        let allocator_resource = self.store.data_mut().create_allocator();
        let allocator = self.store.data_mut().get_allocator_mut(&allocator_resource);
        let arguments = arguments
            .into_iter()
            .map(|v| allocator.allocate_native_value(v))
            .collect::<Vec<_>>();
        let index = module
            .call_call_function(
                &mut self.store,
                self.this,
                name,
                Resource::new_own(allocator_resource.rep()),
                &arguments,
            )
            .expect("There should be no wasm errors")?;
        let allocator = self.store.data_mut().get_allocator_mut(&allocator_resource);
        let value = allocator.get(index);
        let value = allocator.convert_to_native_value(value);
        Ok(value)
    }
}

bindgen!({
    path: "../slides-arrow/wit/world.wit",
    // with: {
    //     "wasi": wasmtime_wasi::bindings,
    // }
});

#[derive(Debug, Clone)]
struct HostValueAllocator {
    index: usize,
    values: Vec<modules::Value>,
}

impl HostValueAllocator {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            values: Vec::new(),
        }
    }

    fn allocate(&mut self, value: arrows::values::Value) -> arrows::values::ValueIndex {
        let index = self.values.len();
        self.values.push(value);
        arrows::values::ValueIndex { index: index as _ }
    }

    fn get(&self, value: arrows::values::ValueIndex) -> arrows::values::Value {
        self.values[value.index as usize].clone()
    }

    fn convert_to_native_value(&self, value: arrows::values::Value) -> value::Value {
        match value {
            arrows::values::Value::Void => value::Value::Void(()),
            arrows::values::Value::StringType(s) => value::Value::String(s),
            arrows::values::Value::Int(i) => value::Value::Integer(i),
            arrows::values::Value::Float(f) => value::Value::Float(f),
            arrows::values::Value::StyleUnit(style_unit) => {
                value::Value::StyleUnit(style_unit.parse().expect("Should not fail"))
            }
            arrows::values::Value::Position(position) => value::Value::Position(Position {
                x: position.x.parse().expect("Should not fail"),
                y: position.y.parse().expect("Should not fail"),
            }),
            arrows::values::Value::Dict(dict) => todo!(),
            arrows::values::Value::Array(items) => todo!(),
        }
    }

    fn allocate_native_value(&mut self, value: value::Value) -> values::ValueIndex {
        match value {
            value::Value::Void(_) => self.allocate(values::Value::Void),
            value::Value::Float(it) => self.allocate(values::Value::Float(it)),
            value::Value::Integer(it) => self.allocate(values::Value::Int(it)),
            value::Value::String(it) => self.allocate(values::Value::StringType(it)),
            value::Value::Dict(hash_map) => {
                let entries = hash_map
                    .into_iter()
                    .map(|(key, value)| {
                        let value = self.allocate_native_value(value);
                        (key, value)
                    })
                    .collect();
                self.allocate(values::Value::Dict(entries))
            }
            value::Value::Array(values) => {
                let values = values
                    .into_iter()
                    .map(|v| self.allocate_native_value(v))
                    .collect();
                self.allocate(values::Value::Array(values))
            }
            _ => todo!("Cannot allocatoe native value!"),
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    value_allocators: Vec<HostValueAllocator>,
}

impl State {
    fn get_allocator_mut(
        &mut self,
        self_: &Resource<arrows::values::ValueAllocator>,
    ) -> &mut HostValueAllocator {
        let index = self_.rep() as usize;
        &mut self.value_allocators[index]
    }
    fn get_allocator(
        &self,
        self_: &Resource<arrows::values::ValueAllocator>,
    ) -> &HostValueAllocator {
        let index = self_.rep() as usize;
        &self.value_allocators[index]
    }

    fn new() -> Self {
        Self {
            value_allocators: Vec::new(),
        }
    }

    fn create_allocator(&mut self) -> wasmtime::component::Resource<values::ValueAllocator> {
        values::HostValueAllocator::create(self)
    }
}

impl arrows::types::Host for State {}
impl arrows::values::Host for State {}

impl arrows::values::HostValueAllocator for State {
    fn create(&mut self) -> wasmtime::component::Resource<arrows::values::ValueAllocator> {
        let index = self.value_allocators.len();
        self.value_allocators.push(HostValueAllocator::new(index));
        Resource::new_own(index as _)
    }

    fn allocate(
        &mut self,
        self_: wasmtime::component::Resource<arrows::values::ValueAllocator>,
        value: modules::Value,
    ) -> arrows::values::ValueIndex {
        let allocator = self.get_allocator_mut(&self_);
        allocator.allocate(value)
    }

    fn get(
        &mut self,
        self_: wasmtime::component::Resource<arrows::values::ValueAllocator>,
        value: arrows::values::ValueIndex,
    ) -> modules::Value {
        let allocator = self.get_allocator(&self_);
        allocator.get(value)
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<arrows::values::ValueAllocator>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

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

    let mut store = Store::new(&engine, State::new());
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
        this: module,
        bindings,
        engine,
        store,
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
                (
                    f.name.clone(),
                    ModuleFunction {
                        name: f.name,
                        type_,
                    },
                )
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

impl From<value::Value> for modules::Value {
    fn from(value: value::Value) -> Self {
        match value {
            value::Value::Void(_) => Self::Void,
            value::Value::String(s) => Self::StringType(s),
            value::Value::Integer(i) => Self::Int(i),
            value::Value::Float(f) => Self::Float(f),
            _ => unreachable!("Unsupported conversion!"),
        }
    }
}
