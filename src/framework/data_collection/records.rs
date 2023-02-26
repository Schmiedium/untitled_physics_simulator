use crate::framework::plugins::base_plugin::WorldTimer;

use crate::framework::py_modules::simulation_builder::{Name, RecordInitializer};
use bevy::prelude::{
    Commands, Component, Entity, EventReader, EventWriter, Query, Res, Resource, Transform,
};
use polars::prelude::{NamedFrom, PolarsResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type DataframeStore = HashMap<String, Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>;

#[derive(Component, Default)]
pub struct Record {
    pub name: String,
    pub dataframes: Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>,
}
#[derive(Resource)]
pub struct DataframeStoreResource(pub DataframeStore);

#[derive(Resource)]
pub struct DataFrameSender(pub flume::Sender<DataframeStore>);

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
    query_entities: Query<(Entity, &Name, &RecordInitializer)>,
    world_timer: Res<WorldTimer>,
) {
    // iterator over all entities found by the query
    for (e, n, r_it) in query_entities.iter() {
        let mut r = Record::default();
        r.name = format!("{}_{}", n.0.clone(), e.index().to_string());

        commands
            .entity(e)
            .insert(
                //create the record to be added
                r,
            )
            .remove::<RecordInitializer>();
    }
}

pub struct UpdateRecordEvent {
    pub record: Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>,
    pub table_name: String,
    pub new_row: polars::frame::DataFrame,
}

pub fn position_update_record_event(
    transforms: Query<(&Transform, &Record)>,
    mut record_updates: EventWriter<UpdateRecordEvent>,
    world_timer: Res<WorldTimer>,
) {
    for (t, record) in transforms.iter() {
        let new_row = polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Position_X" => [t.translation.x], "Position_Y" => [t.translation.y], "Position_Z" => [t.translation.z]].unwrap();
        let table_name = format!("Position");

        record_updates.send(UpdateRecordEvent {
            record: record.dataframes.clone(),
            table_name,
            new_row,
        });
    }
}

pub fn update_record_event_reader(mut record_updates: EventReader<UpdateRecordEvent>) {
    for update in record_updates.iter() {
        match _update_record(update) {
            Ok(_) => {}
            Err(_) => todo!(),
        }
    }
}

pub fn _update_record(update: &UpdateRecordEvent) -> PolarsResult<()> {
    match update.record.write() {
        Ok(mut df) => match df.get_mut(&update.table_name) {
            Some(df) => Ok({
                df.vstack_mut(&update.new_row)?;
            }),
            None => {
                df.insert(update.table_name.clone(), update.new_row.clone());
                Ok(())
            }
        },
        Err(_) => todo!(),
    }
}
