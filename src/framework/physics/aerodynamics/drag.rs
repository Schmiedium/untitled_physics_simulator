use super::common::*;
use crate::framework::ps_component::PSComponent;
use crate::framework::py_modules::entity_builder::Entity;
use bevy::prelude::{Component, FromReflect, Query, Reflect, Transform};
use bevy_rapier3d::dynamics::{ExternalForce, Velocity};
use bevy_rapier3d::math::Real;
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

        let drag_force = get_aero_constant(v, t, &1.0) * drag_coefficient * -v.linvel.normalize();

        f.force = f.force + drag_force;
    }
}

fn get_linear_drag_coefficient() -> Real {
    0.2953
}
