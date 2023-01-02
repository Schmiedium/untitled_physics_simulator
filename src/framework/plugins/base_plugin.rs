use crate::framework::data_collection::records::{
    initialize_records, update_records, DataFrameSender, DataframeStore, Record,
};
use bevy::prelude::*;

use crate::framework::py_modules::simulation_builder;

pub(super) struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<simulation_builder::Name>();
        app.register_type::<simulation_builder::Shape>();
        app.register_type::<simulation_builder::ColliderInitializer>();
        app.register_type::<simulation_builder::RecordInitializer>();
        app.insert_resource(bevy::winit::WinitSettings {
            return_from_run: true,
            ..bevy::prelude::default()
        });
        app.add_system(initialize_records);
        app.add_system(update_records.after(initialize_records));
        app.add_system(advance_world_time);
        app.add_system(exit_system);
    }
}

#[derive(Resource)]
pub struct WorldTimer {
    pub simulation_end_time: bevy_rapier3d::prelude::Real,
    pub timer: bevy::time::Stopwatch,
    pub dt: f32,
}

fn advance_world_time(mut world_timer: ResMut<WorldTimer>) {
    let step = world_timer.dt;
    world_timer
        .timer
        .tick(std::time::Duration::from_secs_f32(step));
}

///
/// # Exit System
/// * system that that determines exit functionality
/// * need to be updated to accept a simulation end time, and quit after
/// * also need to figure out move semantics to avoid the disgusting double clone of all the dataframes
///
///
fn exit_system(
    world_timer: Res<WorldTimer>,
    mut record_components: Query<&mut Record>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut records: ResMut<DataframeStore>,
    sender: Res<DataFrameSender>,
) {
    //Determine if exit criterion is met
    if world_timer.timer.elapsed_secs() > world_timer.simulation_end_time {
        // Iterate over all the record components found by the query
        for r in record_components.iter_mut() {
            // Destructure the record component into name and dataframe variables
            // Clone one is here and need to figure out how to remove it
            let (name, df) = ((r.record_name).to_string(), r.dataframe.clone());

            // insert name and dataframe into the hashmap holding onto the data
            records
                .0
                .insert(name, Box::<polars::frame::DataFrame>::new(df));
        }
        // Clone the resource hashmap into something returnable
        let return_map = records.0.clone();

        // send returnable hashmap back to main thread
        sender.0.send(return_map).unwrap();

        // Send AppExit event to quit the simualtion
        exit.send(bevy::app::AppExit);
    }
}
