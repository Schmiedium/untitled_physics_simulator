use crate::framework::data_collection::records::RecordTrait;
use bevy::{prelude::Component, reflect::Reflect};
use polars::prelude::{NamedFrom, PolarsResult};
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Reflect)]
struct TestModel {}

#[pymethods]
impl TestModel {
    #[new]
    fn new() -> Self {
        TestModel {}
    }
}

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
        let new_row = &polars::df!["Time" => [time], "Value" => [1]].unwrap();
        let k = format!("{}_{}_Position", name, index.to_string());

        match record.dataframes.clone().write() {
            Ok(mut df) => {
                df.get_mut(&k).unwrap().vstack_mut(new_row)?;
                Ok(())
            }
            Err(_) => todo!(),
        }
    }
}
