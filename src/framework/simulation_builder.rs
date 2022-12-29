use std::collections::HashMap;

use bevy::{prelude::{Component, Vec3, Transform}, transform::TransformBundle};
use bevy_rapier3d::prelude::Velocity;
use pyo3::{prelude::*, types::PyTuple};

#[pyclass]
struct Simulation {
    positions: HashMap<&str, TransformBundle>,
    velocities: HashMap<&str, Velocity>,

}

#[pymethods]
impl Simulation {
    #[new]
    fn new() -> Self {
        Simulation {
            positions: HashMap::new(),
            velocities: HashMap::new(),
         }
    }
    
    pub fn create_entity(&mut self, name: String, position: &PyTuple, velocity: &PyTuple, acceleration: &PyTuple) -> PyResult<()>{

        let pos: (f32, f32, f32) = position.extract()?;
        let vel: (f32, f32, f32) = velocity.extract()?;
        let acc: (f32, f32, f32) = acceleration.extract()?;
    
        let trans = TransformBundle::from(Transform::from_xyz(pos.0, pos.1, pos.2));
        let vel_comp: Velocity = Velocity{linvel: Vec3::new(vel.0, vel.1, vel.2), angvel: Vec3::new(0.0,0.0,0.0)};

        self.positions.insert(&*name, trans);
        self.velocities.insert(&*name, vel_comp);
    
        
    
        Ok(())
    }
}
