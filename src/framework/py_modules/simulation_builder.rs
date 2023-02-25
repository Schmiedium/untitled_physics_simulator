use std::{path::PathBuf, sync::Arc};

use crate::models::test::test_model::TestModel;

use super::entity_builder::Entity;
use bevy::{
    prelude::{Component, GlobalTransform, ReflectComponent, Resource, Transform, Vec3},
    reflect::{FromReflect, Reflect, TypeRegistry, TypeRegistryInternal},
    scene::{DynamicEntity, DynamicScene},
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{Real, RigidBody, Velocity};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyTuple};
use serde::de::DeserializeSeed;

#[pyclass]
#[derive(Resource)]
pub struct Simulation {
    pub entity_ids: Vec<u32>,
    pub timestep: Real,
    pub wall_time: Real,
    pub sim_duration: Real,
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
            entity_ids: Vec::new(),
            timestep: self.timestep,
            wall_time: self.wall_time,
            sim_duration: self.sim_duration,
            scene: new_scene,
            types: new_types1,
        }
    }
}

#[pymethods]
impl Simulation {
    #[new]
    fn new(timestep: Real, sim_duration: Real, wall_time: Real) -> Self {
        let mut new_sim = Simulation {
            entity_ids: Vec::new(),
            timestep,
            wall_time,
            sim_duration,
            scene: DynamicScene {
                entities: Vec::new(),
            },
            types: TypeRegistryInternal::new(),
        };

        //Register all types with the type registry
        //necessary for serialization

        //RUST STD TYPES
        new_sim.types.register::<String>();
        new_sim.types.register::<PathBuf>();

        //RAPIER TYPES
        new_sim.types.register::<Transform>();
        new_sim.types.register::<GlobalTransform>();
        new_sim.types.register::<Velocity>();
        new_sim.types.register::<RigidBody>();
        new_sim.types.register::<glam::Quat>();
        new_sim.types.register::<glam::Vec3>();
        new_sim.types.register::<glam::Vec3A>();
        new_sim.types.register::<glam::Affine3A>();
        new_sim.types.register::<glam::Mat3A>();

        //MY CUSTOM TYPES
        new_sim.types.register::<Name>();
        new_sim.types.register::<Shape>();
        new_sim.types.register::<RecordInitializer>();
        new_sim.types.register::<ColliderInitializer>();
        new_sim.types.register::<TestModel>();

        //End registering types

        new_sim
    }

    // Create a basic entity in line and add it to the simulation
    pub fn create_entity(
        &mut self,
        index: u32,
        name: String,
        entity_type: String,
        position: &PyTuple,
        velocity: &PyTuple,
        geometry: String,
    ) -> PyResult<()> {
        //BEGIN Error handling
        if self.entity_ids.contains(&index) {
            return Err(pyo3::exceptions::PyAttributeError::new_err(format!("Index \"{}\" for entity \"{}\" is already in use, overlapping entity indices may cause some entities to get overwritten", index.to_string(), &name)));
        }
        //END Error handling

        self.entity_ids.push(index);

        //BEGIN Setup all necessary components

        //create name components
        let n = Name(name.clone());

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
        let trans_bundle =
            TransformBundle::from_transform(Transform::from_xyz(pos.0, pos.1, pos.2));
        let trans = trans_bundle.local;
        let gtrans = trans_bundle.global;

        //extract velocity vector components from input tuple
        let vel: (f32, f32, f32) = velocity.extract()?;
        //build velocity component
        let vel_comp = Velocity {
            linvel: Vec3::new(vel.0, vel.1, vel.2),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        };

        let ci = ColliderInitializer {
            path: PathBuf::from(geometry),
            shape: Shape::Trimesh,
        };

        //END setting up necessary components

        //Begin boxing all entities for storage in the scene

        let trans_b = Box::new(trans);
        let gtrans_b = Box::new(gtrans);
        let vel_comp_b = Box::new(vel_comp);
        let body_b = Box::new(body);
        let n_b = Box::new(n);
        let ri_b = Box::new(RecordInitializer(name));
        let ci_b = Box::new(ci);

        //End boxing all entities

        //initialize data store for constucting simulation object
        let components: Vec<Box<dyn Reflect>> =
            vec![trans_b, gtrans_b, vel_comp_b, body_b, n_b, ri_b, ci_b];

        let entity = DynamicEntity {
            entity: index,
            components,
        };

        self.scene.entities.push(entity);

        Ok(())
    }

    pub fn add_entity(&mut self, e: Entity) -> PyResult<()> {
        let index = self.get_first_unused_index();

        let mut e1: DynamicEntity = DynamicEntity {
            entity: index,
            components: e.components,
        };

        e1.components.push(Box::new(Name(e.name)));

        self.scene.entities.push(e1);

        self.entity_ids.push(index);

        Ok(())
    }

    pub fn add_entities(&mut self, entities: Vec<Entity>) -> PyResult<()> {
        for e in entities {
            self.add_entity(e)?
        }
        Ok(())
    }

    fn get_first_unused_index(&self) -> u32 {
        let len = self.entity_ids.len() as u32;
        if !self.entity_ids.contains(&len) {
            return len;
        } else {
            0
        }
    }
}

#[derive(Component, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct Name(pub String);

#[derive(Component, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct RecordInitializer(pub String);

impl RecordInitializer {}

#[derive(Component, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct ColliderInitializer {
    pub path: PathBuf,
    pub shape: Shape,
}

#[derive(Reflect, FromReflect, Default)]

pub enum Shape {
    #[default]
    Trimesh,
    Computed,
}

#[derive(Resource)]
pub struct WallTime(pub Real);
