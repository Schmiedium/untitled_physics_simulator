use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use framework::data_collection::records::{DataFrameSender, DataframeStore};
use framework::plugins::base_plugin::WorldTimer;
use framework::{
    data_collection::dataframe_conversions, geometry::geometry_parsing,
    py_modules::simulation_builder,
};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyList};
use std::collections::HashMap;

mod framework;

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
    m.add_function(wrap_pyfunction!(simulation_run_headless, m)?)?;
    m.add_class::<simulation_builder::Simulation>()?;
    Ok(())
}

#[pyfunction]
fn simulation_run(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    let world_timer = WorldTimer {
        simulation_end_time: simulation.sim_duration,
        timer: bevy::time::Stopwatch::new(),
        dt: simulation.timestep,
    };

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

    let dataframes: DataframeStore = DataframeStore(Box::new(HashMap::new()));
    let (sender, receiver) =
        flume::unbounded::<Box<HashMap<String, Box<polars::frame::DataFrame>>>>();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(simulation)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_physics)
        .add_system(initialize_colliders)
        .add_plugins(framework::plugins::plugin_group::UntitledPluginsGroup)
        .run();

    let dfs = *receiver.recv().unwrap();

    if dfs.is_empty() {
        println!("QUACK");
        return Python::with_gil(|py| -> PyResult<PyObject> {
            Ok("no data to return".to_object(py))
        });
    }

    dataframe_hashmap_to_python_dict(dfs)
}

#[pyfunction]
fn simulation_run_headless(simulation: simulation_builder::Simulation) -> PyResult<PyObject> {
    let world_timer = WorldTimer {
        simulation_end_time: simulation.sim_duration,
        timer: bevy::time::Stopwatch::new(),
        dt: simulation.timestep,
    };

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

    let dataframes: DataframeStore = DataframeStore(Box::new(HashMap::new()));
    let (sender, receiver) =
        flume::unbounded::<Box<HashMap<String, Box<polars::frame::DataFrame>>>>();

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(config)
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(simulation)
        .add_startup_system(setup_physics)
        .add_system(initialize_colliders)
        .add_plugins(framework::plugins::plugin_group::UntitledPluginsGroupHeadless)
        .run();

    let dfs = *receiver.recv().unwrap();

    if dfs.is_empty() {
        println!("QUACK");
        return Python::with_gil(|py| -> PyResult<PyObject> {
            Ok("no data to return".to_object(py))
        });
    }

    dataframe_hashmap_to_python_dict(dfs)
}

/// This is not a bevy system, but a function extracted from main for converting the data collected
/// during the sim into a format that can be pass back to python
fn dataframe_hashmap_to_python_dict(dfs: HashMap<String, Box<DataFrame>>) -> PyResult<PyObject> {
    // Closure from HELL
    // takes key, value pair from the dataframes hashmap and returns a tuple of name and python-polars dataframe
    let closure = |item: (String, Box<polars::frame::DataFrame>)| -> PyResult<(String, PyObject)> {
        // destructure input tuple
        let df = *item.1;
        let key = item.0;

        // need to own names of the columns for iterator purposes
        let names = df.get_column_names_owned();

        // bruh i don't event know
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
            // oh it's a collection now, so we have to call into_iterator because we need ownership I think
            .into_iter()
            // oh NOW we can unzip. Look I'm glad this works, but it was really annoying.
            // can't unzip without collecting cause compiler errors. kinda bothers me
            .unzip();

        // This is a python tuple
        // it contains a list of Arrow Series and a List of therir names
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

    let keys_values = dfs
        .into_iter()
        .collect::<Vec<(String, Box<polars::frame::DataFrame>)>>()
        .into_iter()
        .map(closure)
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

    let return_dict = Python::with_gil(|py| -> PyResult<PyObject> {
        Ok((keys_values.into_py_dict(py)).to_object(py))
    });

    return_dict
}

fn setup_camera(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(
    mut commands: Commands,
    input: Res<simulation_builder::Simulation>,
    mut scene: ResMut<Assets<DynamicScene>>,
) {
    /* Create the ground. */
    commands.spawn((
        Collider::cuboid(10000000.0, 0.1, 10000000.0),
        TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
    ));

    // /* Create the bouncing ball. */
    // commands.spawn((
    //     RigidBody::Dynamic,
    //     Collider::ball(0.5),
    //     Restitution::coefficient(0.7),
    //     simulation_builder::Name("Ball".to_string()),
    //     simulation_builder::RecordInitializer,
    //     // Record {
    //     //     record_name: "Ball".to_string(),
    //     //     record_output: true,
    //     //     dataframe: polars::frame::DataFrame::default(),
    //     // },
    //     TransformBundle::from(Transform::from_xyz(0.0, 6.0, 0.0)),
    //     Velocity {
    //         linvel: Vec3::new(0.0, 0.0, 0.0),
    //         angvel: Vec3::new(0.0, 0.0, 0.0),
    //     },
    // ));

    let scene_handle = scene.add(input.to_owned().scene);

    commands.spawn(DynamicSceneBundle {
        scene: scene_handle,
        ..default()
    });
}

fn initialize_colliders(
    mut commands: Commands,
    q: Query<(Entity, &simulation_builder::ColliderInitializer)>,
) {
    for (e, ci) in q.iter() {
        commands
            .entity(e)
            .insert(Restitution::coefficient(0.7))
            .remove::<simulation_builder::ColliderInitializer>();

        match ci.shape {
            simulation_builder::Shape::Trimesh => {
                if let Ok(colliders) = geometry_parsing::parse_obj_into_trimeshes(&ci.path, 1.0) {
                    for c in colliders {
                        commands.entity(e).insert(c);
                    }
                };
            }
            simulation_builder::Shape::Computed => {
                if let Ok(colliders) =
                    geometry_parsing::parse_obj_into_computed_shape(&ci.path, 1.0)
                {
                    for c in colliders {
                        commands.entity(e).insert(c);
                    }
                };
            }
        }
    }
}
