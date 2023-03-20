use crate::framework::physics::aerodynamics::drag::*;
use bevy::app::{App, Plugin};
use bevy::prelude::{IntoSystemConfig, Query, SystemSet, Transform};
use bevy_rapier3d::prelude::ExternalForce;
use glam::Vec3;

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
        // app.add_system(print_rotations);
    }
}
