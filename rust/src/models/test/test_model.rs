use crate::framework::data_collection::records::RecordTrait;
use bevy::{
    prelude::Component,
    reflect::{FromReflect, Reflect},
};
use polars::prelude::NamedFrom;
use pyo3::{pyclass, pymethods, FromPyObject};

#[pyclass]
#[derive(Component, Reflect, FromReflect, FromPyObject)]
struct TestModel {
    test: String,
}

#[pymethods]
impl TestModel {
    #[new]
    fn new() -> Self {
        TestModel {
            test: "Bkah".to_string(),
        }
    }
}

impl RecordTrait for TestModel {
    fn initialize_record(
        &self,
        record: &mut crate::framework::data_collection::records::Record,
        index: u32,
        name: String,
        time: f32,
    ) {
        let first_row = polars::df!["Time" => [time], "Value" => [1]].unwrap();
        let k: String = format!("TestModel");
        record.name = format!("{}_{}", index, &name);

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
    ) -> polars::prelude::PolarsResult<()> {
        let new_row = &polars::df!["Time" => [time], "Value" => [1]].unwrap();
        let k = format!("Position");

        match record.dataframes.clone().write() {
            Ok(mut df) => {
                df.get_mut(&k).unwrap().vstack_mut(new_row)?;
                Ok(())
            }
            Err(_) => todo!(),
        }
    }
}
