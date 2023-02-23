use crate::framework::{
    data_collection::records::{
        initialize_records, update_records, DataFrameSender, DataframeStoreResource, Record,
        RecordTrait,
    },
    geometry::geometry_parsing,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use bevy_trait_query::RegisterExt;

use crate::framework::py_modules::simulation_builder;

pub(super) struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<simulation_builder::Name>();
        app.register_type::<simulation_builder::Shape>();
        app.register_type::<simulation_builder::ColliderInitializer>();
        app.register_type::<simulation_builder::RecordInitializer>();
        app.register_component_as::<dyn RecordTrait, Transform>();
        app.add_startup_system(setup_physics);
        app.add_system(initialize_colliders);
        app.add_system(initialize_records);
        app.add_system(
            update_records
                .after(initialize_records)
                .after(advance_world_time),
        );
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

// Increments the world time by one timestep
fn advance_world_time(mut world_timer: ResMut<WorldTimer>) {
    let step = world_timer.dt;
    world_timer
        .timer
        .tick(std::time::Duration::from_secs_f32(step));
}

///
/// # Exit System
/// * system that that determines exit functionality
/// * I think I figured out the move semantics, I refactored Record to store a Box<dataframe> and I'm
/// * cloning the box instead. Need to validate that, but it should be correct
///
fn exit_system(
    world_timer: Res<WorldTimer>,
    mut record_components: Query<&Record>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut records: ResMut<DataframeStoreResource>,
    sender: Res<DataFrameSender>,
) {
    //Determine if exit criterion is met
    if world_timer.timer.elapsed_secs() > world_timer.simulation_end_time {
        // Iterate over all the record components found by the query
        for r in record_components.iter_mut() {
            // insert name and dataframe into the hashmap holding onto the data
            records.0.insert(r.name.to_owned(), r.dataframes.clone());
        }
        // Clone the resource hashmap into something returnable
        let return_map = records.0.clone();

        // send returnable hashmap back to main thread
        sender.0.send(return_map).unwrap();

        // Send AppExit event to quit the simualtion
        exit.send(bevy::app::AppExit);
    }
}

/// Sets up all the entities specified in the simulation
/// should probably change the name
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

    //grabs the simulation resource, gets the Dynamic scene within, and gets a handle to that scene
    let scene_handle = scene.add(input.to_owned().scene);

    // spawns the scene using the handle above
    commands.spawn(DynamicSceneBundle {
        scene: scene_handle,
        ..default()
    });
}

/// Colliders don't implement Reflect, and therefore cannot be serialized
/// Since cloning the simulation class requires serialization, something else must be done
/// The `ColliderInitializer` carries the information needed to setup a pre-specified collider
/// This system finds all `ColliderInitializer` objects, makes the colliders, adds them to the appropriate entity,
/// and then removes the `ColliderInitializer` component
///
/// This can probably be done better/more efficiently if done with events or something
/// same with RecordInitializer
fn initialize_colliders(
    mut commands: Commands,
    q: Query<(Entity, &simulation_builder::ColliderInitializer)>,
) {
    for (e, ci) in q.iter() {
        commands
            .entity(e)
            .insert(bevy_rapier3d::prelude::Restitution::coefficient(0.7))
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
