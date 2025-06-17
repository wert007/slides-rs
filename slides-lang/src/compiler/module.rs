use std::{collections::HashMap, io::Read, path::PathBuf};

use component::arrows::{self, slides};
use exports::component::arrows::modules;
use slides_rs_core::FilePlacement;
use wasmtime::{
    Store,
    component::{Component, Linker, Resource, ResourceAny, bindgen},
};

use crate::{
    Context, VariableId,
    compiler::binder::typing::{FunctionType, TypeId},
};

use super::{
    binder::{Binder, typing},
    evaluator::value,
};

pub(crate) mod state;
use state::State;
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
    types: HashMap<String, TypeId>,
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
        self.functions.get(name)
    }

    pub fn try_get_type_by_name(&self, name: &str) -> Option<TypeId> {
        self.types.get(name).copied()
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
        let slides = self.store.data_mut().init_slides();
        let index = module
            .call_call_function(
                &mut self.store,
                self.this,
                slides,
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
    path: "../slides-arrow/wit/module.wit",
    // with: {
    //     "wasi": wasmtime_wasi::bindings,
    // }
});

pub fn load_module(
    name: VariableId,
    path: impl Into<PathBuf>,
    _binder: &mut Binder,
    context: &mut Context,
) -> std::io::Result<Module> {
    let path = path.into();
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    // dbg!(archive.file_names().collect::<Vec<_>>());
    let mut file = archive.by_name("slides_arrow.wasm")?;
    let mut buffer = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buffer)?;

    // let contents = std::fs::read(&path).unwrap();
    // assert!(contents.starts_with(b"\0asm"));
    let engine = wasmtime::Engine::default();

    let component = Component::from_binary(&engine, &buffer).unwrap();

    let mut linker = Linker::new(&engine);
    linker.allow_shadowing(true);
    // linker.define_unknown_imports_as_traps(&component).unwrap();
    Host_::add_to_linker(&mut linker, |state: &mut State| state).unwrap();
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();

    let mut store = Store::new(&engine, State::new(context.presentation.clone()));
    let bindings = Host_::instantiate(&mut store, &component, &linker).unwrap();
    let slides = store.data_mut().init_slides();

    let module = bindings
        .component_arrows_modules()
        .module()
        .call_create(&mut store, slides)
        .unwrap();
    bindings
        .component_arrows_modules()
        .module()
        .call_register_types(&mut store, module, Resource::new_own(0))
        .unwrap();
    context.type_interner.add_from_module(
        store.data_mut().type_allocator_mut(),
        &mut context.string_interner,
    );
    let types = store
        .data()
        .type_allocator()
        .get_all_module_types(&context.type_interner, &context.string_interner);
    let functions = bindings
        .component_arrows_modules()
        .module()
        .call_available_functions(&mut store, module, Resource::new_own(0))
        .unwrap();

    Ok(Module {
        this: module,
        bindings,
        engine,
        name,
        types,
        functions: functions
            .into_iter()
            .map(|f| {
                let type_ = FunctionType {
                    min_argument_count: f.args.len(),
                    argument_types: f
                        .args
                        .into_iter()
                        .map(|t| unsafe { TypeId::from_raw(t.index as _) })
                        .collect(),
                    return_type: unsafe { TypeId::from_raw(f.result_type.index as _) },
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
        store,
    })
}

// impl From<modules::Type> for typing::Type {
//     fn from(value: modules::Type) -> Self {
//         match value {
//             arrows::types::Type::Void => Self::Void,
//             arrows::types::Type::String => Self::String,
//             arrows::types::Type::Int => Self::Integer,
//             arrows::types::Type::Float => Self::Float,
//             arrows::types::Type::Dict => Self::DynamicDict,
//             arrows::types::Type::Element => Self::Element,
//             arrows::types::Type::Enum(name) => {
//                 match name.as_str() {
//                     _ => todo!("Unknowon enum: {name}"),
//                 }
//             },
//             arrows::types::Type::EnumDefinition(_) => todo!(),
//         }
//     }
// }

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

impl From<slides::Placement> for FilePlacement {
    fn from(value: slides::Placement) -> Self {
        match value {
            slides::Placement::HtmlHead => Self::HtmlHead,
            slides::Placement::JavascriptInit => Self::JavascriptInit,
            slides::Placement::JavascriptSlideChange => Self::JavascriptSlideChange,
        }
    }
}
