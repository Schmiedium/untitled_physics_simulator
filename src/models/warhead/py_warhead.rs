use bevy::{prelude::Component, reflect::Reflect};
use pscomp_derive::PSComponent;
use pyo3::{pyclass, pymethods};

use crate::framework::ps_component::PSComponent;
use crate::framework::py_modules::entity_builder::Entity;
#[derive(Component, Clone, Reflect, Default, PSComponent)]
#[pyclass(name = "Warhead")]
pub struct Warhead {
    wh_type: WarheadType,
}

#[derive(Reflect, Clone, Default)]
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

    pub fn attach_to_entity(&self, e: &mut Entity) -> pyo3::PyResult<Entity> {
        let res = self.clone()._attach_to_entity(e.to_owned());
        Ok(res)
    }
}
