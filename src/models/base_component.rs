use bevy::reflect::Reflect;
use pyo3::pyclass;

use crate::framework::data_collection::records::RecordTrait;

#[pyclass(subclass)]
pub struct BaseComponent {
    pub c: Box<dyn Reflect>,
}

impl RecordTrait for BaseComponent {
    fn initialize_record(
        &self,
        record: &crate::framework::data_collection::records::Record,
        index: u32,
        name: String,
        time: f32,
    ) {
        todo!()
    }

    fn update_record(
        &self,
        record: &crate::framework::data_collection::records::Record,
        time: f32,
        index: u32,
        name: String,
    ) -> polars::prelude::PolarsResult<()> {
        todo!()
    }
}

impl Clone for BaseComponent {
    fn clone(&self) -> Self {
        Self {
            c: self.c.clone_value(),
        }
    }
}
