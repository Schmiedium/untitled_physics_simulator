use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyList, PyTuple};
use std::collections::HashMap;

mod framework;
mod utility;

#[derive(Component, Default)]
struct Record {
    record_name: String,
    record_output: bool,
    dataframe: polars::frame::DataFrame,
}

/// A Python module implemented in Rust.
#[pymodule]
fn untitled_physics_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(simulation_run, m)?)?;
    Ok(())
}

#[pyfunction]
fn simulation_run() -> PyResult<PyObject> {
    let config = RapierConfiguration {
        gravity: Vect::Y * -9.81,
        physics_pipeline_active: true,
        query_pipeline_active: true,
        timestep_mode: TimestepMode::Fixed {
            dt: 0.001,
            substeps: 1,
        },
        scaled_shape_subdivision: 10,
    };

    let dataframes: Box<HashMap<String, Box<polars::frame::DataFrame>>> = Box::new(HashMap::new());
    let (sender, receiver) =
        flume::unbounded::<Box<HashMap<String, Box<polars::frame::DataFrame>>>>();

    App::new()
        .add_plugins_with(DefaultPlugins, |group| {
            group.disable::<bevy::log::LogPlugin>()
        })
        .insert_resource(config)
        .insert_resource(bevy::winit::WinitSettings {
            return_from_run: true,
            ..default()
        })
        .insert_resource(dataframes)
        .insert_resource(sender)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_camera)
        .add_startup_system(setup_physics)
        .add_startup_system(initialize_records)
        .add_system(update_records)
        .add_system(exit_system)
        .run();

    let dfs = *receiver.recv().unwrap();

    if dfs.is_empty() {
        println!("QUACK");
        return Python::with_gil(|py| -> PyResult<PyObject> {
            Ok("no data to return".to_object(py))
        });
    }

    let closure = |item: (String, Box<polars::frame::DataFrame>)| -> PyResult<(String, PyObject)> {
        let df = *item.1;
        let key = item.0;

        let names = df.get_column_names_owned();

        let thing = df
            .columns(names)
            .unwrap()
            .into_iter()
            .map(|s| -> PyObject { utility::rust_series_to_py_series(s).unwrap() })
            .collect::<Vec<PyObject>>();

        let blah = Python::with_gil(|py| -> PyResult<PyObject> {
            let list: &PyList = PyList::new(py, thing);
            let arg = (list,);

            let pl = py.import("polars")?;
            let maybe_what_i_want = pl.getattr("DataFrame")?;
            let out = maybe_what_i_want.call1(arg)?;
            Ok(out.to_object(py))
        })?;

        Ok((key, blah))
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

    let return_type = Python::with_gil(|py| -> PyResult<PyObject> {
        Ok((keys_values.into_py_dict(py)).to_object(py))
    });

    return_type
}

fn setup_camera(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(Record {
            record_name: "Ball".to_string(),
            record_output: true,
            dataframe: polars::frame::DataFrame::default(),
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)));
}

fn initialize_records(mut record_components: Query<(&mut Record, &Transform)>) {
    for (mut r, t) in record_components.iter_mut() {
        if r.record_output {
            let position = &t.translation;
            let dataframe =
            polars::df!["Position_X" => [position.x], "Position_Y" => [position.y], "Position_Z" => [position.z]].unwrap();
            r.dataframe = dataframe;
        }
    }
}

fn update_records(mut record_components: Query<(&mut Record, &Transform)>) {
    for (mut r, t) in record_components.iter_mut() {
        if r.record_output {
            let position = &t.translation;
            let new_row = polars::df!["Position_X" => [position.x], "Position_Y" => [position.y], "Position_Z" => [position.z]].unwrap();

            r.dataframe.vstack_mut(&new_row).unwrap();
        }
    }
}

fn exit_system(
    time: Res<Time>,
    mut record_components: Query<&mut Record>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut records: ResMut<Box<HashMap<String, Box<polars::frame::DataFrame>>>>,
    sender: Res<flume::Sender<Box<HashMap<String, Box<polars::frame::DataFrame>>>>>,
) {
    if time.seconds_since_startup() > 5.0 {
        for mut r in record_components.iter_mut() {
            let (mut name, df) = ((r.record_name).to_string(), r.dataframe.clone());
            name.push_str(".csv");
            // let mut file = std::fs::File::create(&name).unwrap();
            // CsvWriter::new(&mut file).finish(&mut r.dataframe).unwrap();

            records.insert(name, Box::<polars::frame::DataFrame>::new(df));
        }
        let thing = records.clone();
        sender.send(thing).unwrap();

        exit.send(bevy::app::AppExit);
    }
}
