use crate::framework::plugins::base_plugin::WorldTimer;

use crate::framework::py_modules::simulation_builder::{Name, RecordInitializer};
use bevy::prelude::{Commands, Component, Entity, Query, Res, Resource, Transform};
use polars::prelude::{NamedFrom, PolarsResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Component, Default)]
pub struct Record {
    pub dataframes: Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>,
}

#[bevy_trait_query::queryable]
pub trait RecordTrait {
    fn initialize_record(&self, record: &Record, index: u32, name: String, time: f32) {
        Record::default();
    }

    fn update_record(&self, record: &Record, time: f32, name: String) -> PolarsResult<()> {
        Ok(())
    }
}

impl RecordTrait for Transform {
    fn initialize_record(&self, record: &Record, index: u32, name: String, time: f32) {
        let first_row = polars::df!["Time" => [time], "Position_X" => [self.translation.x], "Position_Y" => [self.translation.y], "Position_Z" => [self.translation.z]].unwrap();

        match record.dataframes.write() {
            Ok(mut rw_guard) => {
                rw_guard.insert(name, first_row);
            }
            Err(_) => todo!(),
        }
    }

    fn update_record(&self, record: &Record, time: f32, name: String) -> PolarsResult<()> {
        let new_row = &polars::df!["Time" => [time], "Position_X" => [self.translation.x], "Position_Y" => [self.translation.y], "Position_Z" => [self.translation.z]].unwrap();

        match record.dataframes.clone().write() {
            Ok(mut df) => {
                df.get_mut(&name).unwrap().vstack_mut(new_row)?;
                Ok(())
            }
            Err(_) => todo!(),
        }
    }
}

#[derive(Resource)]
pub struct DataframeStore(pub Vec<Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>);

#[derive(Resource)]
pub struct DataFrameSender(
    pub flume::Sender<Vec<Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>>,
);

/// Record components don't implement Reflect, and therefore cannot be serialized
/// Since cloning the simulation class requires serialization, something else must be done
/// The RecordInitializer carries the information needed to setup a record
/// This system finds all RecordInitializer objects, makes the record components, adds them to the appropriate entity,
/// and then removes the RecordInitializer component
///
/// This can probably be done better/more efficiently if done with events or something
/// same with ColliderInitializer
///
/// allow unused variables is here the recordinitializer is a unit struct
#[allow(unused_variables)]
pub fn initialize_records(
    mut commands: Commands,
    query_entities: Query<(Entity, &Name, &RecordInitializer, &dyn RecordTrait)>,
    world_timer: Res<WorldTimer>,
) {
    // iterator over all entities found by the query
    for (e, n, r_i, t) in query_entities.iter() {
        let mut r = Record::default();

        t.iter().for_each(|c| {
            c.initialize_record(
                &mut r,
                e.index(),
                n.0.clone(),
                world_timer.timer.elapsed_secs(),
            )
        });

        commands
            .entity(e)
            .insert(
                //create the record to be added
                r,
            )
            .remove::<RecordInitializer>();
    }
}

/// system to query for all entities that have both Record and Transform components
/// grabs data from the transform and records it to the record component
pub fn update_records(
    records: Query<(Entity, &Name, &Record, &dyn RecordTrait)>,
    world_timer: Res<WorldTimer>,
) {
    for (e, name, record, rt) in records.iter() {
        for recording_component in rt.iter() {
            match recording_component.update_record(
                record,
                world_timer.timer.elapsed_secs(),
                name.0.clone(),
            ) {
                Ok(_) => {}
                Err(_) => continue,
            }
        }
    }
}
