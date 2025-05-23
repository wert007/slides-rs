use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use slides_rs_core::{Position, Presentation, WebRenderable};
use wasmtime::component::Resource;
use wasmtime_wasi::{IoView, WasiView};

use crate::compiler::evaluator::value;

use super::{
    component::arrows::{self, slides, values},
    exports::component::arrows::modules,
};

#[derive(Debug, Clone)]
pub struct HostValueAllocator {
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

    pub fn allocate(&mut self, value: arrows::values::Value) -> arrows::values::ValueIndex {
        let index = self.values.len();
        self.values.push(value);
        arrows::values::ValueIndex { index: index as _ }
    }

    pub fn get(&self, value: arrows::values::ValueIndex) -> arrows::values::Value {
        self.values[value.index as usize].clone()
    }

    pub fn convert_to_native_value(&self, value: arrows::values::Value) -> value::Value {
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
            arrows::values::Value::Dict(_dict) => todo!("dict to value"),
            arrows::values::Value::Array(_items) => todo!("array to value"),
            arrows::values::Value::Element(_element) => todo!("element to value"),
        }
    }

    pub fn allocate_native_value(&mut self, value: value::Value) -> values::ValueIndex {
        match value {
            value::Value::Void(_) => self.allocate(values::Value::Void),
            value::Value::Float(it) => self.allocate(values::Value::Float(it)),
            value::Value::Integer(it) => self.allocate(values::Value::Int(it)),
            value::Value::String(it) => self.allocate(values::Value::StringType(it)),
            value::Value::Color(color) => {
                self.allocate(values::Value::StringType(color.to_string()))
            }
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
            value::Value::Element(element) => {
                let element = values::Element {
                    id: element.id().raw() as u32,
                    parent: element.parent().map(|p| p.raw() as u32),
                    name: element.name(),
                    namespace: element.namespace(),
                };
                self.allocate(values::Value::Element(element))
            }
            value::Value::CustomElement(element) => {
                let element = values::Element {
                    id: element.id().raw() as u32,
                    parent: element.parent().map(|p| p.raw() as u32),
                    name: element.name(),
                    namespace: element.namespace(),
                };
                self.allocate(values::Value::Element(element))
            }
            value => todo!("Cannot allocate native value! {value:#?}"),
        }
    }
}

pub struct State {
    value_allocators: Vec<HostValueAllocator>,
    wasi_ctx: wasmtime_wasi::WasiCtx,
    wasi_table: wasmtime_wasi::ResourceTable,
    presentation: Arc<RwLock<Presentation>>,
}

impl State {
    pub fn get_allocator_mut(
        &mut self,
        self_: &Resource<arrows::values::ValueAllocator>,
    ) -> &mut HostValueAllocator {
        let index = self_.rep() as usize;
        &mut self.value_allocators[index]
    }
    pub fn get_allocator(
        &self,
        self_: &Resource<arrows::values::ValueAllocator>,
    ) -> &HostValueAllocator {
        let index = self_.rep() as usize;
        &self.value_allocators[index]
    }

    pub fn new(presentation: Arc<RwLock<Presentation>>) -> Self {
        Self {
            value_allocators: Vec::new(),
            wasi_ctx: wasmtime_wasi::WasiCtxBuilder::new().build(),
            wasi_table: wasmtime_wasi::ResourceTable::default(),
            presentation,
        }
    }

    pub fn create_allocator(&mut self) -> wasmtime::component::Resource<values::ValueAllocator> {
        values::HostValueAllocator::create(self)
    }

    pub fn init_slides(&mut self) -> Resource<arrows::slides::Slides> {
        Resource::new_own(0)
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
        _rep: wasmtime::component::Resource<arrows::values::ValueAllocator>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl slides::Host for State {}

impl slides::HostSlides for State {
    fn download_file(
        &mut self,
        _self_: wasmtime::component::Resource<modules::Slides>,
        url: wasmtime::component::__internal::String,
        path: wasmtime::component::__internal::String,
    ) -> () {
        let path = PathBuf::from(path);
        if path.exists() {
            return;
        }
        let response = reqwest::blocking::get(url.clone())
            .expect("Add error handling")
            .error_for_status()
            .expect("Add error handling");
        let bytes = response.bytes().expect("Add error handling");
        std::fs::write(path, bytes).expect("Add error handling");
    }

    fn add_file_reference(
        &mut self,
        _self_: wasmtime::component::Resource<modules::Slides>,
        path: wasmtime::component::__internal::String,
    ) -> () {
        self.presentation.write().unwrap().add_referenced_file(path);
    }

    fn place_text_in_output(
        &mut self,
        _self_: wasmtime::component::Resource<modules::Slides>,
        text: wasmtime::component::__internal::String,
        source: wasmtime::component::__internal::String,
        placement: arrows::slides::Placement,
    ) -> () {
        self.presentation
            .write()
            .unwrap()
            .add_extern_text(
                placement.into(),
                slides_rs_core::ExternText::Text(source, text),
            )
            .expect("No error can happen here");
    }

    fn drop(
        &mut self,
        _rep: wasmtime::component::Resource<modules::Slides>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi_ctx
    }
}

impl IoView for State {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.wasi_table
    }
}
