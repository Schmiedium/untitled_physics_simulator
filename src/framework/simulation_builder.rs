use std::sync::Arc;

use bevy::{
    prelude::{Component, Resource, Transform, Vec3},
    reflect::{FromReflect, Reflect, TypeRegistry, TypeRegistryInternal},
    scene::{DynamicEntity, DynamicScene},
};
use bevy_rapier3d::prelude::{RigidBody, Velocity};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyTuple};
use serde::de::DeserializeSeed;

#[pyclass]
#[derive(Resource)]
pub struct Simulation {
    pub scene: DynamicScene,
    pub types: TypeRegistryInternal,
}

impl Clone for Simulation {
    fn clone(&self) -> Self {
        let mut new_types1 = TypeRegistryInternal::new();
        for registered_type in self.types.iter() {
            new_types1.add_registration(registered_type.clone());
        }

        let mut new_types2 = TypeRegistryInternal::new();
        for registered_type in self.types.iter() {
            new_types2.add_registration(registered_type.clone());
        }

        let reg_arc = TypeRegistry {
            internal: Arc::new(parking_lot::RwLock::new(new_types2)),
        };

        let scene_ser = self.scene.serialize_ron(&reg_arc).unwrap();

        let scene_deserializer = bevy::scene::serde::SceneDeserializer {
            type_registry: &new_types1,
        };
        let mut deserializer = ron::de::Deserializer::from_str(&scene_ser).unwrap();

        let new_scene = scene_deserializer.deserialize(&mut deserializer).unwrap();

        Self {
            scene: new_scene,
            types: new_types1,
        }
    }
}

#[pymethods]
impl Simulation {
    #[new]
    fn new() -> Self {
        Simulation {
            scene: DynamicScene {
                entities: Vec::new(),
            },
            types: TypeRegistryInternal::new(),
        }
    }

    #[args()]
    pub fn create_entity(
        &mut self,
        index: u32,
        name: String,
        entity_type: String,
        position: &PyTuple,
        velocity: &PyTuple,
    ) -> PyResult<()> {
        //BEGIN Setup all necessary components

        //create name components
        let n = Name(name);

        //match input to supported RigidBody type, return error if invalid
        let body = match &*entity_type {
            //Dyanmic entity will be acted on by gravity/other forces and potentially collide
            "Dynamic" => RigidBody::Dynamic,
            //Fixed entity will be locked in one position
            "Fixed" => RigidBody::Fixed,
            s => {
                return Err(PyValueError::new_err(format!(
                    "entity_type must be either Dynamic or Fixed, {} is invalid",
                    s
                )))
            }
        };

        //extract position vector components from input tuple
        let pos: (f32, f32, f32) = position.extract()?;
        //build transform component bundle to handle position
        let trans = Transform::from_xyz(pos.0, pos.1, pos.2);

        //extract velocity vector components from input tuple
        let vel: (f32, f32, f32) = velocity.extract()?;
        //build velocity component
        let vel_comp = Velocity {
            linvel: Vec3::new(vel.0, vel.1, vel.2),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        };

        //END setting up necessary components

        //Begin boxing all entities for storage in the scene

        let trans_b = Box::new(trans);
        let vel_comp_b = Box::new(vel_comp);
        let body_b = Box::new(body);
        let n_b = Box::new(n);

        //End boxing all entities

        //Register all types with the type registry
        //necessary for serialization

        self.types.register::<Transform>();
        self.types.register::<Velocity>();
        self.types.register::<RigidBody>();
        self.types.register::<Name>();

        //End registering types

        //initialize data store for constucting simulation object
        let mut components: Vec<Box<dyn Reflect>> = Vec::new();

        components.push(trans_b);
        components.push(vel_comp_b);
        components.push(body_b);
        components.push(n_b);

        let entity = DynamicEntity {
            entity: index,
            components: components,
        };

        self.scene.entities.push(entity);

        Ok(())
    }
}

#[derive(Component, Reflect, FromReflect)]
pub struct Name(String);
