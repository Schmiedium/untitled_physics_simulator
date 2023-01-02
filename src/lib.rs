use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use framework::{
    data_collection::dataframe_conversions, geometry::geometry_parsing,
    py_modules::simulation_builder,
};
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyList};
use std::collections::HashMap;
use std::path::PathBuf;

mod framework;

#[derive(Component, Default)]
struct Record {
    record_name: String,
    record_output: bool,
    dataframe: polars::frame::DataFrame,
}

#[derive(Resource)]
struct WorldTimer {
    simulation_end_time: Real,
    timer: bevy::time::Stopwatch,
    dt: f32,
}
#[derive(Resource)]
struct DataframeStore(Box<HashMap<String, Box<polars::frame::DataFrame>>>);

#[derive(Resource)]
struct DataFrameSender(flume::Sender<Box<HashMap<String, Box<polars::frame::DataFrame>>>>);

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .register_type::<simulation_builder::Name>()
        .register_type::<simulation_builder::ColliderInitializer>()
        .register_type::<simulation_builder::RecordInitializer>()
        .insert_resource(config)
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: true,
            ..default()
        })
        .insert_resource(dataframes)
        .insert_resource(DataFrameSender(sender))
        .insert_resource(world_timer)
        .insert_resource(simulation)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_physics)
        .add_system(initialize_records)
        .add_system(initialize_colliders)
        .add_system(advance_world_time)
        .add_system(update_records.after(initialize_records))
        .add_system(exit_system)
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

fn advance_world_time(mut world_timer: ResMut<WorldTimer>) {
    let step = world_timer.dt;
    world_timer
        .timer
        .tick(std::time::Duration::from_secs_f32(step));
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
    println!("entered setup_physics");
    /* Create the ground. */
    commands.spawn((
        Collider::cuboid(10000000.0, 0.1, 10000000.0),
        TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
    ));

    /* Create the bouncing ball. */
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        simulation_builder::Name("Ball".to_string()),
        simulation_builder::RecordInitializer,
        // Record {
        //     record_name: "Ball".to_string(),
        //     record_output: true,
        //     dataframe: polars::frame::DataFrame::default(),
        // },
        TransformBundle::from(Transform::from_xyz(0.0, 6.0, 0.0)),
        Velocity {
            linvel: Vec3::new(0.0, 0.0, 0.0),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        },
    ));

    let scene_handle = scene.add(input.to_owned().scene);

    commands.spawn(DynamicSceneBundle {
        scene: scene_handle,
        ..default()
    });
}

#[allow(unused_variables)]
fn initialize_records(
    mut commands: Commands,
    query_entities: Query<(
        Entity,
        &simulation_builder::Name,
        &simulation_builder::RecordInitializer,
        &Transform,
    )>,
    world_timer: Res<WorldTimer>,
) {
    for (e, n, r_i, t) in query_entities.iter() {
        // Get reference to position from the transform
        let position = &t.translation;
        // create new row to add to the dataframe
        let new_row = polars::df!["Time" => [world_timer.timer.elapsed_secs()], "Position_X" => [position.x], "Position_Y" => [position.y], "Position_Z" => [position.z]].unwrap();

        commands
            .entity(e)
            .insert(Record {
                record_name: n.0.clone(),
                record_output: true,
                dataframe: new_row,
            })
            .remove::<simulation_builder::RecordInitializer>();
    }
}

fn initialize_colliders(
    mut commands: Commands,
    q: Query<(Entity, &simulation_builder::ColliderInitializer)>,
) {
    for (e, c_i) in q.iter() {
        let path = PathBuf::from(&*c_i.0.clone());

        if let Ok(colliders) = geometry_parsing::parse_obj_into_trimesh(path, 1.0) {
            commands
                .entity(e)
                .insert(Restitution::coefficient(0.7))
                .remove::<simulation_builder::ColliderInitializer>();
            for c in colliders {
                commands.entity(e).insert(c);
            }
        };
    }
}

/// system to query for all entities that have both Record and Transform components
/// grabs data from the transform and records it to the record component
fn update_records(
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
