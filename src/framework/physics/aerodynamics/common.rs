use bevy::prelude::Transform;
use bevy_rapier3d::{dynamics::Velocity, math::Real};
use glam::Vec3;

/// Returns the standard reference area given a reference diameter
/// as described in Exterior Ballistics, Robert L McCoy, Chapter 2
fn reference_area(ref_diameter: &Real) -> Real {
    std::f32::consts::PI * ref_diameter * ref_diameter / 4.0
}

fn get_air_density(pos: &Transform) -> Real {
    1.2931
}

pub(in super::super::aerodynamics) fn get_angle_of_attack(v: &Velocity, t: &Transform) -> Real {
    let aoa = v
        .linvel
        .angle_between(Vec3::from(t.rotation.to_scaled_axis()));

    if aoa.is_nan() {
        return 0.0;
    }

    // println!("{}", aoa);

    aoa
}

pub(in super::super::aerodynamics) fn get_aero_constant(
    v: &Velocity,
    t: &Transform,
    d: &Real,
) -> Real {
    let rho = get_air_density(t);
    let ref_area = reference_area(d);

    let vel_vec = v.linvel;
    let vel_mag: Real = vel_vec.length();

    let aero_constant = 0.5 * rho * ref_area * vel_mag.powf(2.0);
    aero_constant
}
