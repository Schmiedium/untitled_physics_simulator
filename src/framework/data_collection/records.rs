use crate::framework::plugins::base_plugin::WorldTimer;

use crate::framework::py_modules::simulation_builder::{Name, RecordInitializer};
use bevy::prelude::{Commands, Component, Entity, Query, Res, Resource, Transform};
use polars::prelude::NamedFrom;
use std::collections::HashMap;

#[derive(Component, Default)]
pub struct Record {
    pub record_name: String,
    pub record_output: bool,
    pub dataframe: Box<polars::frame::DataFrame>,
}

#[derive(Resource)]
pub struct DataframeStore(pub HashMap<String, Box<polars::frame::DataFrame>>);

#[derive(Resource)]
pub struct DataFrameSender(pub flume::Sender<HashMap<String, Box<polars::frame::DataFrame>>>);

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
        // Get reference to position from the transform
        let position = &t.translation;
        // create new row to add to the dataframe
        let new_row = polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Position_X" => [position.x], "Position_Y" => [position.y], "Position_Z" => [position.z]].unwrap();

        commands
            .entity(e)
            .insert(
                //create the record to be added
                Record {
                    record_name: n.0.clone(),
                    record_output: true,
                    dataframe: Box::new(new_row),
                },
            )
            .remove::<RecordInitializer>();
    }
}

/// system to query for all entities that have both Record and Transform components
/// grabs data from the transform and records it to the record component
pub fn update_records(
    mut record_components: Query<(&mut Record, &Transform)>,
    world_timer: Res<WorldTimer>,
) {
    // iterate over query results, destructuring the returned tuple
    for (mut r, t) in record_components.iter_mut() {
        // check if we should be recording data for this record component
        if r.record_output {
            // Get reference to position from the transform
            let position = &t.translation;
            // create new row to add to the dataframe
            let new_row = polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Position_X" => [position.x], "Position_Y" => [position.y], "Position_Z" => [position.z]].unwrap();

            // Call vstack_mut function with the newly created row to append to dataframe
            r.dataframe.vstack_mut(&new_row).unwrap();
        }
    }
}
