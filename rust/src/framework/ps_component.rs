use super::py_modules::entity_builder::Entity;
use bevy::{
    prelude::Component,
    reflect::{FromReflect, Reflect},
};

pub trait PSComponent: Component + Reflect + FromReflect + Default + Clone {
    fn attach_to_entity(self, e: &mut Entity) {
        e.components.push(Box::new(self))
    }
}
