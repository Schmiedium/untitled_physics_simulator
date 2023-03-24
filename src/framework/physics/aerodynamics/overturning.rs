use crate::framework::physics::aerodynamics::common::{get_aero_constant, get_angle_of_attack};
use bevy::prelude::{Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;

pub(super) fn overturning_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v.linvel.cross(t.rotation.to_scaled_axis()).normalize();

        let overturning_moment = get_aero_constant(v, t, &1.0)
            * 1.0
            * get_overturning_moment_coefficient()
            * get_angle_of_attack(v, t).sin()
            * direction_vec;

        f.torque = f.torque + overturning_moment;
    }
}

fn get_overturning_moment_coefficient() -> Real {
    2.88
}
