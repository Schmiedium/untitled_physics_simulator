use bevy::{
    prelude::Component,
    reflect::{FromReflect, Reflect},
};
use pyo3::{pyclass, pymethods};

use crate::framework::ps_component::PSComponent;

#[pyclass]
#[derive(Component, Clone, Reflect, FromReflect, Default)]
pub struct Warhead {
    wh_type: WarheadType,
}

#[derive(Reflect, FromReflect, Clone, Default)]
enum WarheadType {
    #[default]
    Frag,
    Blast,
}

#[pymethods]
impl Warhead {
    #[new]
    pub fn new(_type: String) -> Self {
        Warhead {
            wh_type: WarheadType::Frag,
        }
    }
}

impl PSComponent for Warhead {
    fn attach_to_entity(self, e: &mut crate::framework::py_modules::entity_builder::Entity) {
        e.components.push(Box::new(self))
    }
}
