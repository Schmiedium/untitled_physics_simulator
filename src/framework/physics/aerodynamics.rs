use bevy::app::{App, Plugin};
use bevy::prelude::Query;
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;
use glam::Vec3;

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
fn drag_force(mut ext_forces: Query<&mut ExternalForce>) {
    for f in ext_forces.iter_mut() {
        // Declaring Constants
        // Yes there are a lot

        // Air density
        let rho = 0.0;
        let vel_vec = Vec3::default();
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let zero_yaw_drag_coefficient = 0.0;
        let yaw_drag_coefficient = 0.0;

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);
        let drag_coefficient = zero_yaw_drag_coefficient + yaw_drag_coefficient * delta.powf(2.0);

        // Declaring the resultant force vector that will be added to our "external force" object
        let mut drag_force =
            -0.5 * rho * ref_area * vel_mag.powf(2.0) * drag_coefficient * direction_vec;

        f.force = f.force + drag_force;
    }
}

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
fn magnus_force(mut ext_forces: Query<&mut ExternalForce>) {
    for f in ext_forces.iter_mut() {}
}

fn lift_force(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn pitch_damping_force(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn spin_damping_moment(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn rolling_moment(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn overturning_moment(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn magnus_moment(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

fn pitch_damping_moment(mut ext_forces: Query<&mut ExternalForce>) {
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

/// Returns the standard reference area given a reference diameter
/// as described in Exterior Ballistics, Robert L McCoy, Chapter 2
fn reference_area(ref_diameter: Real) -> Real {
    std::f32::consts::PI * ref_diameter * ref_diameter / 4.0
}

pub struct AerodynamicsPlugin;

impl Plugin for AerodynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(drag_force);
        app.add_system(magnus_force);
        app.add_system(lift_force);
        app.add_system(pitch_damping_force);
        app.add_system(spin_damping_moment);
        app.add_system(rolling_moment);
        app.add_system(overturning_moment);
        app.add_system(magnus_moment);
        app.add_system(pitch_damping_moment);
    }
}
