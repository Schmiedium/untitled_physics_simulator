use crate::framework::data_collection::records::{Record, UpdateRecordEvent};
use crate::framework::physics::aerodynamics::drag::*;
use crate::framework::physics::aerodynamics::lift::lift_force;
use crate::framework::physics::aerodynamics::magnus::{magnus_force, magnus_moment};
use crate::framework::physics::aerodynamics::overturning::overturning_moment;
use crate::framework::physics::aerodynamics::pitch_damping::{
    pitch_damping_force, pitch_damping_moment,
};
use crate::framework::physics::aerodynamics::rolling::rolling_moment;
use crate::framework::physics::aerodynamics::spin_damping::spin_damping_moment;
use crate::framework::plugins::base_plugin::WorldTimer;
use bevy::app::{App, Plugin};
use bevy::prelude::{EventWriter, IntoSystemConfigs, Query, Res, SystemSet, Transform};
use bevy_rapier3d::prelude::{ExternalForce, Velocity};
use glam::Vec3;
use polars::{df, prelude::NamedFrom};

#[derive(SystemSet, Clone, Hash, Eq, PartialEq, Debug)]
struct AeroForceSystems;

fn reset_external_forces(mut forces: Query<&mut ExternalForce>) {
    for mut f in forces.iter_mut() {
        f.force = Vec3::default();
        f.torque = Vec3::default();
    }
}

#[allow(dead_code)]
fn print_rotations(rot: Query<&Transform>) {
    for r in rot.iter() {
        println!("{}", r.rotation);
    }
}

fn record_more_physics(
    physics: Query<(&Transform, &Velocity, &Record)>,
    mut record_updates: EventWriter<UpdateRecordEvent>,
    world_timer: Res<WorldTimer>,
) {
    for (t, v, r) in physics.iter() {
        let new_row = df!["Time" => [world_timer.timer.elapsed_secs()], 
            "Orientation_X" => [t.rotation.to_scaled_axis().x], "Orientation_Y" => [t.rotation.to_scaled_axis().y], "Orientation_Z" => [t.rotation.to_scaled_axis().z], "Orientation_Angle" => [t.rotation.w],
            "Velocity_X" => [v.linvel.x], "Velocity_Y" => [v.linvel.y], "Velocity_Z" => [v.linvel.z], "Velocity_Mag" => [v.linvel.length()],
            "Angular_Velocity_X" => [v.angvel.x], "Angular_Velocity_Y" => [v.angvel.y], "Angular_Velocity_Z" => [v.angvel.z], "Angular_Velocity_Mag" => [v.angvel.length()]
        ].unwrap();
        let table_name = format!("Physics");

        record_updates.send(UpdateRecordEvent {
            record: r.dataframes.clone(),
            table_name,
            new_row,
        });
    }
}

pub struct AerodynamicsPlugin;

impl Plugin for AerodynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DragCurve>();
        app.add_system(drag_force.in_set(AeroForceSystems));
        app.add_system(magnus_force.in_set(AeroForceSystems));
        app.add_system(lift_force.in_set(AeroForceSystems));
        app.add_system(pitch_damping_force.in_set(AeroForceSystems));
        app.add_system(spin_damping_moment.in_set(AeroForceSystems));
        app.add_system(rolling_moment.in_set(AeroForceSystems));
        app.add_system(overturning_moment.in_set(AeroForceSystems));
        app.add_system(magnus_moment.in_set(AeroForceSystems));
        app.add_system(pitch_damping_moment.in_set(AeroForceSystems));
        app.add_system(reset_external_forces.before(AeroForceSystems));
        app.add_system(record_more_physics);
        // app.add_system(print_rotations);
    }
}
