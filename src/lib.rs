use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use framework::data_collection::records::{DataFrameSender, DataframeStoreResource};
use framework::plugins::base_plugin::WorldTimer;
use framework::py_modules::entity_builder;
use framework::py_modules::simulation_builder::WallTime;
use framework::{data_collection::dataframe_conversions, py_modules::simulation_builder};
use models::warhead::py_warhead;
use polars::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;

mod framework;
mod models;

type DataframeStore = HashMap<String, Arc<RwLock<HashMap<String, polars::frame::DataFrame>>>>;

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    //add functions to this module
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
    m.add_function(wrap_pyfunction!(simulation_run_headless, m)?)?;
    //add the simulation builder class
    m.add_class::<simulation_builder::Simulation>()?;
    m.add_class::<entity_builder::Entity>()?;
    m.add_class::<py_warhead::Warhead>()?;
    m.add_class::<crate::models::test::test_model::TestModel>()?;
    m.add_class::<crate::models::gun::gun::Gun>()?;

    Ok(())
}

#[pyfunction]
fn simulation_run(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    // Create world timer resource
    // This will interact with the timing system to make sure we advance and stop
    // with the correct timings
    let (world_timer, config) = setup_sim_resources(simulation.sim_duration, simulation.timestep);

    // DataframeStore is a tuple struct with one element, this facilitates getting output data from bevy
    // once the app is done running
    let dataframes: DataframeStoreResource = DataframeStoreResource(HashMap::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) = flume::unbounded::<DataframeStore>();

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
        .insert_resource(WallTime(simulation.wall_time))
        .insert_resource(simulation)
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: true,
            ..bevy::prelude::default()
        })
        //setting up the camera for rendering
        .add_startup_system(framework::camera::spawn_camera)
        .add_system(framework::camera::pan_orbit_camera)
        // see setup_physics function for details
        // this should be redone with events or something
        .run();

    // receiver from earlier checks if anything has been sent, panics if there is no sender (usually means app didn't run)
    let dfs = receiver.recv().unwrap();

    // call function to convert our hashmap into a python dictionary usable in python, and return it
    dataframe_conversions::dataframe_hashmap_to_python_dict(dfs)
}

#[pyfunction]
fn simulation_run_headless(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    // Create world timer resource
    // This will interact with the timing system to make sure we advance and stop
    // with the correct timinings

    let (world_timer, config) = setup_sim_resources(simulation.sim_duration, simulation.timestep);

    // DataframeStore is a tuple struct with one element, this facilitates getting output data from bevy
    // once the app is done running
    let dataframes: DataframeStoreResource = DataframeStoreResource(HashMap::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) = flume::unbounded::<DataframeStore>();

    // Instantiation of the app. This is the bevy app that will run rapier and everything else
    // This sets up and runs the sim
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(framework::plugins::plugin_group::UntitledPluginsGroupHeadless)
        // .insert_resouce(bevy::tasks::TaskPoolBuilder)
        .insert_resource(config)
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(WallTime(simulation.wall_time))
        .insert_resource(simulation)
        .init_resource::<bevy::window::Windows>()
        .add_asset::<bevy::render::mesh::Mesh>()
        .insert_resource(bevy::app::ScheduleRunnerSettings {
            run_mode: bevy::app::RunMode::Loop { wait: None },
        })
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
