use bevy::{prelude::Transform, reflect::Reflect, transform::TransformBundle};
use bevy_rapier3d::prelude::Velocity;
use glam::Vec3;
use pyo3::{pyclass, pymethods, types::PyTuple, PyObject, PyResult};

#[pyclass]
pub struct Entity {
    pub name: String,
    pub components: Vec<Box<dyn Reflect>>,
}

#[pymethods]
impl Entity {
    #[new]
    fn new() -> Self {
        Entity {
            name: String::default(),
            components: Vec::new(),
        }
    }

    fn add_component(&mut self, component: PyObject) {
        todo!()
    }

    fn add_transform(&mut self, position: &PyTuple) -> PyResult<()> {
        //extract position vector components from input tuple
        let pos: (f32, f32, f32) = position.extract()?;
        //build transform component bundle to handle position
        let trans_bundle =
            TransformBundle::from_transform(Transform::from_xyz(pos.0, pos.1, pos.2));
        let trans = trans_bundle.local;
        let gtrans = trans_bundle.global;

        self.components.push(Box::new(trans));
        self.components.push(Box::new(gtrans));

        Ok(())
    }

    fn add_velocity(&mut self, velocity: &PyTuple) -> PyResult<()> {
        //extract velocity vector components from input tuple
        let vel: (f32, f32, f32) = velocity.extract()?;
        //build velocity component
        let vel_comp = Velocity {
            linvel: Vec3::new(vel.0, vel.1, vel.2),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        };

        self.components.push(Box::new(vel_comp));

        Ok(())
    }
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        let mut new_comp_vec = Vec::new();

        for item in &self.components {
            new_comp_vec.push(item.clone_value());
        }

        Self {
            name: self.name.clone(),
            components: new_comp_vec,
        }
    }
}
