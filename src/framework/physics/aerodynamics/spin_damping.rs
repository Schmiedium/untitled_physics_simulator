use crate::framework::physics::aerodynamics::common::get_aero_constant;
use bevy::prelude::{Query, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;
use glam::Vec3;

pub(super) fn spin_damping_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let spin_damping_moment = calculate_spin_damping_moment(v, t);

        f.torque = f.torque + spin_damping_moment;
    }
}

fn calculate_spin_damping_moment(v: &Velocity, t: &Transform) -> Vec3 {
    let direction_vec = -v.angvel.normalize();

    let spin_damping_moment = get_aero_constant(v, t, &1.0)
        * get_spin_damping_moment_coefficient()
        * 1.0
        * 1.0
        * v.angvel.length()
        * v.linvel.length_recip()
        * direction_vec;
    spin_damping_moment
}

fn get_spin_damping_moment_coefficient() -> Real {
    0.003
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Transform;
    use bevy_rapier3d::prelude::Velocity;
    use glam::{Quat, Vec3};

    #[test]
    fn spin_damping_moment_test() {
        let mut v = Velocity::linear(Vec3::new(49.969540, 0.0, 0.0));
        v.angvel = Vec3::new(12.558719, 0.438560, 0.0);
        let mut t = Transform::from_translation(Vec3::new(0.099935, 3.003460, 0.0));
        t.rotation = Quat::from_scaled_axis(Vec3::new(3.164799, 0.110453, 0.0));
        assert!(
            (calculate_spin_damping_moment(&v, &t).normalize() - v.angvel.normalize()).x < 0.000001
        );
        assert!(
            (calculate_spin_damping_moment(&v, &t).normalize() - v.angvel.normalize()).y < 0.000001
        );
        assert!(
            (calculate_spin_damping_moment(&v, &t).normalize() - v.angvel.normalize()).z < 0.000001
        );
    }
}
