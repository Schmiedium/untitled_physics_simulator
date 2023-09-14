use crate::framework::physics::aerodynamics::common::get_aero_constant;
use bevy::prelude::{Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;

pub(super) fn rolling_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let fin_cant_angle = 0.0;

        let direction_vec = -v.angvel.normalize();

        let rolling_moment = get_aero_constant(v, t, &1.0)
            * get_rolling_moment_coefficient()
            * 1.0
            * fin_cant_angle
            * direction_vec;

        f.torque = f.torque + rolling_moment;
    }
}

fn get_rolling_moment_coefficient() -> Real {
    0.0
}
