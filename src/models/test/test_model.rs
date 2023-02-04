use crate::{
    framework::data_collection::records::RecordTrait, models::base_component::BaseComponent,
};
use polars::prelude::{NamedFrom, PolarsResult};
use pyo3::pyclass;

#[pyclass(extends=BaseComponent, subclass)]
struct TestModel {}

impl RecordTrait for TestModel {
    fn initialize_record(
        &self,
        record: &crate::framework::data_collection::records::Record,
        index: u32,
        name: String,
        time: f32,
    ) {
        let first_row = polars::df!["Time" => [time], "Value" => [1]].unwrap();
        let k: String = format!("{}_{}_TestModel", index, &name);

        match record.dataframes.write() {
            Ok(mut rw_guard) => {
                rw_guard.insert(k, first_row);
            }
            Err(_) => todo!(),
        }
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
