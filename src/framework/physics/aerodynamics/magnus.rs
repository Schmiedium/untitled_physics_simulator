use crate::framework::physics::aerodynamics::common::{get_aero_constant, get_angle_of_attack};
use bevy::prelude::{Mut, Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;
use glam::Vec3;

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
pub(super) fn magnus_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let magnus_force = calculate_magnus_force(v, t);

        f.force = f.force + magnus_force;
    }
}

pub(super) fn magnus_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let magnus_moment = calculate_magnus_moment(v, t);

        f.torque = f.torque + magnus_moment;
    }
}

fn calculate_magnus_force(v: &Velocity, t: &Transform) -> Vec3 {
    let direction_vec = v.linvel.cross(t.rotation.to_scaled_axis()).normalize();

    let magnus_force = get_aero_constant(v, t, &1.0)
        * get_magnus_force_coefficient()
        * get_angle_of_attack(v, t)
        * v.angvel.length()
        * v.linvel.length_recip()
        * 1.0
        * direction_vec;

    magnus_force
}

fn calculate_magnus_moment(v: &Velocity, t: &Transform) -> Vec3 {
    let direction_vec = t
        .rotation
        .to_scaled_axis()
        .cross(v.linvel.cross(t.rotation.to_scaled_axis()))
        .normalize();

    let magnus_moment = get_aero_constant(v, t, &1.0)
        * 1.0
        * 1.0
        * v.linvel.length_recip()
        * v.angvel.length()
        * get_magnus_moment_coefficient()
        * get_angle_of_attack(v, t).sin()
        * direction_vec;

    magnus_moment
}

fn get_magnus_force_coefficient() -> Real {
    -0.01
}

fn get_magnus_moment_coefficient() -> Real {
    0.05
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Transform;
    use bevy_rapier3d::prelude::Velocity;
    use glam::{Quat, Vec3};

    #[test]
    fn magnus_moment_test() {
        let mut v = Velocity::linear(Vec3::new(49.969540, 0.0, 0.0));
        v.angvel = Vec3::new(12.558719, 0.438560, 0.0);
        let mut t = Transform::from_translation(Vec3::new(0.099935, 3.003460, 0.0));
        t.rotation = Quat::from_scaled_axis(Vec3::new(3.164799, 0.110453, 0.0));
        assert!((calculate_magnus_moment(&v, &t).normalize() - v.angvel.normalize()).x < 0.000001);
        assert!((calculate_magnus_moment(&v, &t).normalize() - v.angvel.normalize()).y < 0.000001);
        assert!((calculate_magnus_moment(&v, &t).normalize() - v.angvel.normalize()).z < 0.000001);
    }

    #[test]
    fn magnus_force_test() {
        let mut v = Velocity::linear(Vec3::new(49.969540, 0.0, 0.0));
        v.angvel = Vec3::new(12.558719, 0.438560, 0.0);
        let mut t = Transform::from_translation(Vec3::new(0.099935, 3.003460, 0.0));
        t.rotation = Quat::from_scaled_axis(Vec3::new(3.164799, 0.110453, 0.0));
        assert!((calculate_magnus_force(&v, &t).normalize() - v.angvel.normalize()).x < 0.000001);
        assert!((calculate_magnus_force(&v, &t).normalize() - v.angvel.normalize()).y < 0.000001);
        assert!((calculate_magnus_force(&v, &t).normalize() - v.angvel.normalize()).z < 0.000001);
    }
}
