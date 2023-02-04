use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use framework::data_collection::records::{DataFrameSender, DataframeStore};
use framework::plugins::base_plugin::WorldTimer;
use framework::py_modules::entity_builder;
use framework::{data_collection::dataframe_conversions, py_modules::simulation_builder};
use polars::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;

mod framework;
mod models;

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    //add functions to this module
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
    m.add_function(wrap_pyfunction!(simulation_run_headless, m)?)?;
    //add the simulation builder class
    m.add_class::<simulation_builder::Simulation>()?;
    m.add_class::<entity_builder::Entity>()?;

    Ok(())
}

#[pyfunction]
fn simulation_run(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    // Create world timer resource
    // This will interact with the timing system to make sure we advance and stop
    // with the correct timinings
    let (world_timer, config) = setup_sim_resources(simulation.sim_duration, simulation.timestep);

    // DataframeStore is a tuple struct with one element, this facilitates getting output data from bevy
    // once the app is done running
    let dataframes: DataframeStore = DataframeStore(Vec::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) =
        flume::unbounded::<Vec<Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>>();

    // Instantiation of the app. This is the bevy app that will run rapier and everything else
    // This sets up and runs the sim
    App::new()
        .add_plugins(DefaultPlugins)
        // custom plugins group for organization, this was getting unwieldy without it
        .add_plugins(framework::plugins::plugin_group::UntitledPluginsGroup)
        // inserting all the resources created above
        .insert_resource(config)
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(simulation)
        //setting up the camera for rendering
        .add_startup_system(setup_camera)
        // see setup_physics function for details
        // this should be redone with events or something
        .run();

    // receiver from earlier checks if anything has been sent, panincs if there is no sender (usually means app didn't run)
    let dfs = receiver.recv().unwrap();

    // call function to convert our hashmap into a python dictionary usable in python, and return it
    dataframe_conversions::dataframe_hashmap_to_python_dict(dfs)
}

/// Sets up a camera to view whatever is rendering
fn setup_camera(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

#[pyfunction]
fn simulation_run_headless(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    // Create world timer resource
    // This will interact with the timing system to make sure we advance and stop
    // with the correct timinings

    let (world_timer, config) = setup_sim_resources(simulation.sim_duration, simulation.timestep);

    // DataframeStore is a tuple struct with one element, this facilitates getting output data from bevy
    // once the app is done running
    let dataframes: DataframeStore = DataframeStore(Vec::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) =
        flume::unbounded::<Vec<Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>>();

    // Instantiation of the app. This is the bevy app that will run rapier and everything else
    // This sets up and runs the sim
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(config)
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(simulation)
        .add_plugins(framework::plugins::plugin_group::UntitledPluginsGroupHeadless)
        .run();

    let dfs = receiver.recv().unwrap();

    dataframe_conversions::dataframe_hashmap_to_python_dict(dfs)
}

fn setup_sim_resources(sim_duration: f32, timestep: f32) -> (WorldTimer, RapierConfiguration) {
    let world_timer = WorldTimer {
        simulation_end_time: sim_duration,
        timer: bevy::time::Stopwatch::new(),
        dt: timestep,
    };

    //Rapier is the physics engine, this sets up the configuration resource for that engine
    let config = RapierConfiguration {
        gravity: Vect::Y * -9.81,
        physics_pipeline_active: true,
        query_pipeline_active: true,
        timestep_mode: TimestepMode::Fixed {
            dt: world_timer.dt,
            substeps: 1,
        },
        scaled_shape_subdivision: 10,
        force_update_from_transform_changes: default(),
    };

    (world_timer, config)
}
