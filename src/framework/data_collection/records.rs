use crate::framework::plugins::base_plugin::WorldTimer;

use crate::framework::py_modules::simulation_builder::{Name, RecordInitializer};
use bevy::prelude::{Commands, Component, Entity, Query, Res, Resource, Transform};
use polars::prelude::{NamedFrom, PolarsResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Component, Default)]
pub struct Record {
    pub record_name: String,
    pub dataframe: Arc<RwLock<polars::frame::DataFrame>>,
}

#[bevy_trait_query::queryable]
pub trait RecordTrait {
    fn initialize_record(&self, index: u32, name: String, time: f32) -> Record {
        Record::default()
    }

    fn update_record(
        &self,
        record: Arc<RwLock<polars::frame::DataFrame>>,
        time: f32,
    ) -> PolarsResult<()> {
        Ok(())
    }
}

impl RecordTrait for Transform {
    fn initialize_record(&self, index: u32, name: String, time: f32) -> Record {
        Record {
            record_name: format!("Position for: {}_{}", name, index),
            dataframe: Arc::new(RwLock::new(polars::df!["Time" => [time], "Position_X" => [self.translation.x], "Position_Y" => [self.translation.y], "Position_Z" => [self.translation.z]].unwrap())),
        }
    }

    fn update_record(
        &self,
        record: Arc<RwLock<polars::frame::DataFrame>>,
        time: f32,
    ) -> PolarsResult<()> {
        let new_row = &polars::df!["Time" => [time], "Position_X" => [self.translation.x], "Position_Y" => [self.translation.y], "Position_Z" => [self.translation.z]].unwrap();

        match record.write() {
            Ok(mut df) => {
                df.vstack_mut(new_row)?;
                Ok(())
            }
            Err(_) => todo!(),
        }
    }
}

#[derive(Resource)]
pub struct DataframeStore(pub HashMap<String, Arc<RwLock<polars::frame::DataFrame>>>);

#[derive(Resource)]
pub struct DataFrameSender(
    pub flume::Sender<HashMap<String, Arc<RwLock<polars::frame::DataFrame>>>>,
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
    query_entities: Query<(Entity, &Name, &RecordInitializer, &Transform)>,
    world_timer: Res<WorldTimer>,
) {
    // iterator over all entities found by the query
    for (e, n, r_i, t) in query_entities.iter() {
        commands
            .entity(e)
            .insert(
                //create the record to be added
                t.initialize_record(e.index(), n.0.clone(), world_timer.timer.elapsed_secs()),
            )
            .remove::<RecordInitializer>();
    }
}

/// system to query for all entities that have both Record and Transform components
/// grabs data from the transform and records it to the record component
pub fn update_records(records: Query<(&Record, &dyn RecordTrait)>, world_timer: Res<WorldTimer>) {
    for (record, rt) in records.iter() {
        for recording_component in rt.iter() {
            match recording_component
                .update_record(record.dataframe.clone(), world_timer.timer.elapsed_secs())
            {
                Ok(_) => {}
                Err(_) => continue,
            }
        }
    }
}
