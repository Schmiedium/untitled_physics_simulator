use crate::framework::{
    data_collection::records::RecordTrait, ps_component::PSComponent,
    py_modules::entity_builder::Entity,
};
use bevy::{
    prelude::{Component, ReflectComponent},
    reflect::{FromReflect, Reflect},
};
use polars::prelude::NamedFrom;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Clone, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct TestModel {
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

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        println!("attached Test Model to entity");
        Ok(res)
    }
}

impl RecordTrait for TestModel {
    fn initialize_record(
        &self,
        record: &mut crate::framework::data_collection::records::Record,
        time: f32,
    ) {
        let first_row = polars::df!["Time" => [time], "Value" => [1]].unwrap();
        let k: String = format!("TestModel");

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
        let k = format!("TestModel");

        match record.dataframes.clone().write() {
            Ok(mut df) => {
                df.get_mut(&k).unwrap().vstack_mut(new_row)?;
                Ok(())
            }
            Err(_) => todo!(),
        }
    }
}

impl PSComponent for TestModel {
    fn _attach_to_entity(self, mut e: Entity) -> Entity {
        e.components.push(Box::new(self));
        e
    }
}
