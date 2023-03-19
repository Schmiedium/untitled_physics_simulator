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
        // Air density
        let rho = get_air_density(t);
        let ref_area = reference_area(1.0);

        let vel_vec = v.linvel;
        let vel_mag: Real = vel_vec.length();
        let drag_force_direction = -vel_vec.normalize();

        let zero_yaw_drag_coefficient = get_linear_drag_coefficient();
        let yaw_drag_coefficient = 0.0;

        let total_yaw = t.rotation.xyz().angle_between(vel_vec);

        println!("yaw is: {:?}", total_yaw);

        let delta: Real = total_yaw.sin();
        // ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);
        let drag_coefficient = zero_yaw_drag_coefficient;
        // + yaw_drag_coefficient * delta.powf(2.0);

        println!("drag_coefficient is: {:?}", drag_coefficient);

        // Declaring the resultant force vector that will be added to our "external force" object
        let drag_force =
        // aero blah stuff * force coefficient * force vector
            -0.5 * rho * ref_area * vel_mag.powf(2.0) * drag_coefficient * drag_force_direction;

        println!("drag force is happening with magnitude: {:?}", &drag_force);

        f.force = f.force + drag_force;
    }
}

/// Applies the drag force to an entity. Has a number of assumptions that I need to enumerate
pub(super) fn magnus_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut f, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let linear_magnus_coefficient = 0.0;
        let cubic_magnus_coefficient = 0.0;
        let magnus_coefficient =
            linear_magnus_coefficient + cubic_magnus_coefficient * delta.powf(2.0);

        let magnus_force =
            0.5 * rho * vel_mag.powf(2.0) * ref_area * magnus_coefficient * delta * direction_vec;

        f.force = f.force + magnus_force;
    }
}

pub(super) fn lift_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut f, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);
        let linear_lift_coefficient = 0.0;
        let cubic_lift_coefficient = 0.0;
        let lift_coefficient = linear_lift_coefficient + cubic_lift_coefficient * delta.powf(2.0);

        let lift_force =
            0.5 * rho * vel_mag.powf(2.0) * ref_area * lift_coefficient * delta * direction_vec;

        f.force = f.force + lift_force;
    }
}

pub(super) fn pitch_damping_force(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut f, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let pitch_damping_force = Vec3::default();

        f.force = f.force + pitch_damping_force;
    }
}

pub(super) fn spin_damping_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut t, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let spin_damping_moment = Vec3::default();

        t.torque = t.torque + spin_damping_moment;
    }
}

pub(super) fn rolling_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut t, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let rolling_moment = Vec3::default();

        t.torque = t.torque + rolling_moment;
    }
}

pub(super) fn overturning_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut t, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let overturning_moment = Vec3::default();

        t.torque = t.torque + overturning_moment;
    }
}

pub(super) fn magnus_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut t, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag2: Real = vel_vec.length_squared();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let magnus_moment = Vec3::default();

        t.torque = t.torque + magnus_moment;
    }
}

pub(super) fn pitch_damping_moment(mut ext_forces: Query<(&mut ExternalForce, &Velocity)>) {
    for (mut t, v) in ext_forces.iter_mut() {
        let rho = 0.0;
        let ref_diam = 0.0;
        let ref_area = reference_area(ref_diam);
        let vel_vec = v.linvel;
        let direction_vec = vel_vec.normalize();
        let vel_mag: Real = vel_vec.length();

        // Angle of attack - pitch
        let alpha: Real = 0.0;

        // Angle of sideslip - yaw
        let beta: Real = 0.0;
        let delta: Real = ((alpha.sin() * beta.cos()).powf(2.0) + (beta.sin().powf(2.0))).powf(0.5);

        let pitch_damping_moment = Vec3::default();

        t.torque = t.torque + pitch_damping_moment;
    }
}

/// Returns the standard reference area given a reference diameter
/// as described in Exterior Ballistics, Robert L McCoy, Chapter 2
fn reference_area(ref_diameter: Real) -> Real {
    std::f32::consts::PI * ref_diameter * ref_diameter / 4.0
}

fn dynamic_pressure(vel: &Real) -> Real {
    let rho = 0.0751;
    0.5 * rho * vel * vel
}

fn get_air_density(pos: &Transform) -> Real {
    0.0751
}

fn get_linear_drag_coefficient() -> Real {
    0.2953
}
