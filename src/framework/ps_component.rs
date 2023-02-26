use super::py_modules::entity_builder::Entity;
use bevy::{
    prelude::Component,
    reflect::{FromReflect, Reflect},
};

/// This can be derived if all trait bounds are satisfied and enables all the functionality the API
/// expects
pub trait PSComponent: Component + Reflect + FromReflect + Default + Clone {
    /// This function gets derived and needs a method in a #[pymethods]
    /// impl block to call it so that entity::add_component function works properly

    fn _attach_to_entity(self, mut e: Entity) -> Entity {
        e.components.push(Box::new(self));
        e
    }
}
