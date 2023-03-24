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

#[cfg(tests)]
mod tests {
    use crate::framework::physics::aerodynamics::spin_damping::calculate_spin_damping_moment;
    use bevy::prelude::Transform;
    use bevy_rapier3d::prelude::Velocity;
    use glam::{Quat, Vec3};

    #[test]
    fn test1() {
        let mut v = Velocity::angular(Vec3::ZERO);
        v.angvel = Vec3::ZERO;
        let mut t = Transform::from_translation(Vec3::ZERO);
        t.rotation = Quat::zeroed();
        assert_eq!(calculate_spin_damping_moment(&v, &t) == Vec3::ZERO);
    }
}
