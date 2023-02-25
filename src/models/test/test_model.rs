use crate::framework::{
    data_collection::records::{Record, UpdateRecordEvent},
    plugins::base_plugin::WorldTimer,
    ps_component::PSComponent,
    py_modules::entity_builder::Entity,
};
use bevy::{
    prelude::{Component, EventWriter, Query, ReflectComponent, Res},
    reflect::{FromReflect, Reflect},
};
use polars::prelude::NamedFrom;
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Clone, Reflect, FromReflect, Default, PSComponent)]
#[reflect(Component)]
pub struct TestModel {
    test: String,
}

#[pymethods]
impl TestModel {
    #[new]
    fn new(name: String) -> Self {
        TestModel { test: name }
    }

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}

pub fn test_model_update_record_event(
    transforms: Query<(&TestModel, &Record)>,
    mut record_updates: EventWriter<UpdateRecordEvent>,
    world_timer: Res<WorldTimer>,
) {
    for (t, record) in transforms.iter() {
        let new_row =
            polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Value" => [t.test.clone()]]
                .unwrap();
        let table_name = format!("TestModel");

        record_updates.send(UpdateRecordEvent {
            record: record.dataframes.clone(),
            table_name,
            new_row,
        });
    }
}
