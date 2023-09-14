use crate::framework::physics::aerodynamics::common::{get_aero_constant, get_angle_of_attack};
use bevy::prelude::{Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;

pub(super) fn lift_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v
            .linvel
            .cross(t.rotation.to_scaled_axis().cross(v.linvel))
            .normalize();

        let lift_force = get_aero_constant(v, t, &1.0)
            * get_lift_force_coefficient()
            * get_angle_of_attack(v, t).sin()
            * direction_vec;

        f.force = f.force + lift_force;
    }
}

fn get_lift_force_coefficient() -> Real {
    2.69
}
