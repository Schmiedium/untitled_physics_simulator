use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use framework::data_collection::records::{DataFrameSender, DataframeStore};
use framework::plugins::base_plugin::WorldTimer;
use framework::{data_collection::dataframe_conversions, py_modules::simulation_builder};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyList};
use std::collections::HashMap;

mod framework;

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    //add functions to this module
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
    m.add_function(wrap_pyfunction!(simulation_run_headless, m)?)?;
    //add the simulation builder class
    m.add_class::<simulation_builder::Simulation>()?;
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
    let dataframes: DataframeStore = DataframeStore(HashMap::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) = flume::unbounded::<HashMap<String, Arc<polars::frame::DataFrame>>>();

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
    dataframe_hashmap_to_python_dict(dfs)
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
    let dataframes: DataframeStore = DataframeStore(HashMap::new());

    // Create flume sender and receiver, for sending data between threads
    // Bevy is designed to be super parallel, so something like this is necessary
    // The sender goes inside the app, and will send the Hashmap of dataframes back out on simulation exit
    // The receiver stays here and will receive the sent data
    let (sender, receiver) = flume::unbounded::<HashMap<String, Arc<polars::frame::DataFrame>>>();

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

    dataframe_hashmap_to_python_dict(dfs)
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

/// This is not a bevy system, but a function extracted from main for converting the data collected
/// during the sim into a format that can be pass back to python
fn dataframe_hashmap_to_python_dict(dfs: HashMap<String, Arc<DataFrame>>) -> PyResult<PyObject> {
    if dfs.is_empty() {
        println!("QUACK");
        return Python::with_gil(|py| -> PyResult<PyObject> {
            Ok("no data to return".to_object(py))
        });
    }

    // This is a somewhat arcane closure, which will be passed to a map function later
    // takes key, value pair from the dataframes hashmap and returns a tuple of name and python-polars dataframe
    let closure = |item: (String, Arc<polars::frame::DataFrame>)| -> PyResult<(String, PyObject)> {
        // destructure input tuple
        let df = &*item.1;
        let key = item.0;

        // need to own names of the columns for iterator purposes
        let names = df.get_column_names_owned();

        // something about iterating over the dataframe to turn it into Apache Arrow Series and column names as Strings
        let (arrows_series_list, names_list): (Vec<PyObject>, Vec<String>) = df
            // generate Vec of Apache Arrow Series from dataframe object
            .columns(&names)
            // unwrap to handle errors. in the future should handle appropriately, but for now will always work
            .unwrap()
            // turn Vec of Apache Arrow Series into an iterator
            .into_iter()
            // generate iterater over tuples of Series with their respective names
            .zip(names.into_iter())
            // convert rust Series to python Series
            .map(|(s, n)| -> (PyObject, String) {
                (
                    //this function was copied was copied from reddit/stackoverflow/github
                    dataframe_conversions::rust_series_to_py_series(s).unwrap(),
                    n,
                )
            })
            //gotta collect the output into a collection before we turn it into the tuple we want
            .collect::<Vec<(PyObject, String)>>()
            // It's a collection now, so we have to call into_iterator because we need ownership I think
            .into_iter()
            // unzip into the data structure we want
            .unzip();

        // This is a python tuple
        // it contains a list of Arrow Series and a List of their names
        let returning_frame = Python::with_gil(|py| -> PyResult<PyObject> {
            let arg = (
                PyList::new(py, arrows_series_list),
                PyList::new(py, names_list),
            );

            // making sure the python environment has polars
            let pl = py.import("polars")?;
            //construct polars DataFrame from Series and their names
            let out = pl.call_method1("DataFrame", arg)?;

            //Return Python formatted valid dataframe
            Ok(out.to_object(py))
        })?;

        Ok((key, returning_frame))
    };

    //End arcane closure

    // iterate over the hashmap passed in and return a python dictionary of names and dataframes
    let keys_values = dfs
        .into_iter()
        .collect::<Vec<(String, Arc<polars::frame::DataFrame>)>>()
        .into_iter()
        // call map with the arcane closure above
        .map(closure)
        // map that result to the interior of the result, as a python object
        .map(|py_res| -> (String, PyObject) {
            match py_res {
                Ok(x) => x,
                Err(e) => {
                    let object: Py<PyAny> = Python::with_gil(|py| {
                        e.print(py);
                        "quack".to_string().to_object(py)
                    });
                    ("failure to return dataframe".to_string(), object)
                }
            }
        })
        .collect::<Vec<(String, PyObject)>>();

    // construct python dictionary object, and then return it
    Python::with_gil(|py| -> PyResult<PyObject> {
        Ok((keys_values.into_py_dict(py)).to_object(py))
    })
}
