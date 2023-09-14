use crate::framework::physics::aerodynamics::common::get_aero_constant;
use bevy::prelude::{Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;

pub(super) fn pitch_damping_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let geometric_axis_vector = t.rotation.to_scaled_axis().normalize();
        let spin_rotational_velocity_vector =
            v.angvel.dot(geometric_axis_vector) / v.angvel.length() * geometric_axis_vector;
        let tranverse_angular_velocity = v.angvel - spin_rotational_velocity_vector;

        let direction_vec = geometric_axis_vector
            .cross(tranverse_angular_velocity)
            .normalize();

        // NOT CORRECT
        // MUST FIGURE HOW TO SUBTRACT OUT SPIN ROTATIONAL VELOCITY

        let pitch_damping_moment = get_aero_constant(v, t, &1.0)
            * 1.0
            * 1.0
            * v.linvel.length_recip()
            * tranverse_angular_velocity.length()
            * get_pitch_damping_moment_coefficient()
            * direction_vec;

        f.torque = f.torque + pitch_damping_moment;
    }
}

pub(super) fn pitch_damping_force(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let geometric_axis_vector = t.rotation.to_scaled_axis().normalize();
        let spin_rotational_velocity_vector =
            v.angvel.dot(geometric_axis_vector) / v.angvel.length() * geometric_axis_vector;
        let transverse_angular_velocity = v.angvel - spin_rotational_velocity_vector;

        let direction_vec = transverse_angular_velocity.normalize();

        // NOT CORRECT
        // MUST FIGURE HOW TO SUBTRACT OUT SPIN ROTATIONAL VELOCITY

        let pitch_damping_force = get_aero_constant(v, t, &1.0)
            * 1.0
            * 1.0
            * v.linvel.length_recip()
            * transverse_angular_velocity.length()
            * get_pitch_damping_force_coefficient()
            * direction_vec;

        f.force = f.force + pitch_damping_force;
    }
}

fn get_pitch_damping_force_coefficient() -> Real {
    0.004
}

fn get_pitch_damping_moment_coefficient() -> Real {
    -5.5
}
