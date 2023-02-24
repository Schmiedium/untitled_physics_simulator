use super::py_modules::entity_builder::Entity;
use bevy::{
    prelude::Component,
    reflect::{FromReflect, Reflect},
};

pub trait PSComponent: Component + Reflect + FromReflect + Default + Clone {
    fn _attach_to_entity(self, mut e: Entity) -> Entity {
        e.components.push(Box::new(self));
        e
    }
}
