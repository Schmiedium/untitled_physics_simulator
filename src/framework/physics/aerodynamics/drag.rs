use crate::framework::ps_component::PSComponent;
use crate::framework::py_modules::entity_builder::Entity;
use bevy::prelude::{Component, FromReflect, Query, Reflect, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;
use glam::Vec3;
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Component, Reflect, FromReflect, Clone, Default, PSComponent)]
pub struct DragCurve {
    cd: Vec<f32>,
    mach_number: Vec<f32>,
}

#[pymethods]
impl DragCurve {
    #[new]
    pub fn new() -> Self {
        return DragCurve {
            cd: vec![0.2938, 0.2938],
            mach_number: vec![0.0, 5.0],
        };
    }

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
pub(super) fn drag_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let drag_coefficient = get_linear_drag_coefficient();

        let drag_force = get_aero_constant(v, t, 1.0) * drag_coefficient * -v.linvel.normalize();

        f.force = f.force + drag_force;
    }
}

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
pub(super) fn magnus_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v.linvel.cross(t.rotation.to_scaled_axis()).normalize();

        let magnus_force = get_aero_constant(v, t, 1.0)
            * get_magnus_force_coefficient()
            * get_angle_of_attack(v, t)
            * v.angvel.length()
            * v.linvel.length_recip()
            * 1.0
            * direction_vec;

        f.force = f.force + magnus_force;
    }
}

pub(super) fn lift_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v
            .linvel
            .cross(t.rotation.to_scaled_axis().cross(v.linvel))
            .normalize();

        let lift_force = get_aero_constant(v, t, 1.0)
            * get_lift_force_coefficient()
            * get_angle_of_attack(v, t).sin()
            * direction_vec;

        f.force = f.force + lift_force;
    }
}

pub(super) fn pitch_damping_force(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v.angvel.normalize();

        // NOT CORRECT
        // MUST FIGURE HOW TO SUBTRACT OUT SPIN ROTATIONAL VELOCITY

        let pitch_damping_force = get_aero_constant(v, t, 1.0)
            * 1.0
            * 1.0
            * v.linvel.length_recip()
            // * v.angvel.length()
            * get_pitch_damping_force_coefficient()
            * direction_vec;

        f.force = f.force + pitch_damping_force;
    }
}

pub(super) fn spin_damping_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = -v.angvel.normalize();

        let spin_damping_moment = get_aero_constant(v, t, 1.0)
            * get_spin_damping_moment_coefficient()
            * 1.0
            * 1.0
            * v.angvel.length()
            * v.linvel.length_recip()
            * direction_vec;

        f.torque = f.torque + spin_damping_moment;
    }
}

pub(super) fn rolling_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let fin_cant_angle = 0.0;

        let direction_vec = -v.angvel.normalize();

        let rolling_moment = get_aero_constant(v, t, 1.0)
            * get_rolling_moment_coefficient()
            * 1.0
            * fin_cant_angle
            * direction_vec;

        f.torque = f.torque + rolling_moment;
    }
}

pub(super) fn overturning_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = v.linvel.cross(t.rotation.to_scaled_axis()).normalize();

        let overturning_moment = get_aero_constant(v, t, 1.0)
            * 1.0
            * get_overturning_moment_coefficient()
            * get_angle_of_attack(v, t).sin()
            * direction_vec;

        f.torque = f.torque + overturning_moment;
    }
}

pub(super) fn magnus_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = t
            .rotation
            .to_scaled_axis()
            .cross(v.linvel.cross(t.rotation.to_scaled_axis()))
            .normalize();

        let magnus_moment = get_aero_constant(v, t, 1.0)
            * 1.0
            * 1.0
            * v.linvel.length_recip()
            * v.angvel.length()
            * get_magnus_moment_coefficient()
            * get_angle_of_attack(v, t).sin()
            * direction_vec;

        f.torque = f.torque + magnus_moment;
    }
}

pub(super) fn pitch_damping_moment(
    mut ext_forces: Query<(&mut ExternalForce, &Velocity, &Transform)>,
) {
    for (mut f, v, t) in ext_forces.iter_mut() {
        let direction_vec = t.rotation.to_scaled_axis().cross(v.angvel.normalize());

        // NOT CORRECT
        // MUST FIGURE HOW TO SUBTRACT OUT SPIN ROTATIONAL VELOCITY

        let pitch_damping_moment = get_aero_constant(v, t, 1.0)
            * 1.0
            * 1.0
            * v.linvel.length_recip()
            // * v.angvel.length()
            * get_pitch_damping_moment_coefficient()
            * direction_vec;

        f.torque = f.torque + pitch_damping_moment;
    }
}

/// Returns the standard reference area given a reference diameter
/// as described in Exterior Ballistics, Robert L McCoy, Chapter 2
fn reference_area(ref_diameter: Real) -> Real {
    std::f32::consts::PI * ref_diameter * ref_diameter / 4.0
}

fn get_air_density(pos: &Transform) -> Real {
    0.0751
}

fn get_angle_of_attack(v: &Velocity, t: &Transform) -> Real {
    let aoa = v
        .linvel
        .angle_between(Vec3::from(t.rotation.to_scaled_axis()));

    if aoa.is_nan() {
        return 0.0;
    }

    // println!("{}", aoa);

    aoa
}

fn get_aero_constant(v: &Velocity, t: &Transform, d: Real) -> Real {
    let rho = get_air_density(t);
    let ref_area = reference_area(d);

    let vel_vec = v.linvel;
    let vel_mag: Real = vel_vec.length();

    let aero_constant = 0.5 * rho * ref_area * vel_mag.powf(2.0);
    aero_constant
}

fn get_linear_drag_coefficient() -> Real {
    0.2953
}

fn get_lift_force_coefficient() -> Real {
    2.69
}

fn get_magnus_force_coefficient() -> Real {
    -0.01
}

fn get_magnus_moment_coefficient() -> Real {
    0.05
}

fn get_overturning_moment_coefficient() -> Real {
    2.88
}

fn get_pitch_damping_force_coefficient() -> Real {
    0.004
}

fn get_pitch_damping_moment_coefficient() -> Real {
    -5.5
}

fn get_spin_damping_moment_coefficient() -> Real {
    0.003
}

fn get_rolling_moment_coefficient() -> Real {
    0.0
}
