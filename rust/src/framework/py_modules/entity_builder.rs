use std::path::PathBuf;

use bevy::{prelude::Transform, reflect::Reflect, transform::TransformBundle};
use bevy_rapier3d::prelude::{RigidBody, Velocity};
use glam::Vec3;
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyObject, PyResult};

use super::simulation_builder::{ColliderInitializer, RecordInitializer, Shape};

#[pyclass]
pub struct Entity {
    pub name: String,
    pub components: Vec<Box<dyn Reflect>>,
}

#[pymethods]
impl Entity {
    #[new]
    fn new(entity_type: String, name: String) -> PyResult<Self> {
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

        let mut e = Entity {
            name: name.clone(),
            components: Vec::new(),
        };

        e.components.push(Box::new(body));
        e.components.push(Box::new(RecordInitializer(name)));

        Ok(e)
    }

    fn add_component(&mut self, component: PyObject) -> PyResult<Self> {
        let res = pyo3::Python::with_gil(|py| -> PyResult<Self> {
            match component.call_method1(py, "attach_to_entity", (self.clone(),)) {
            Ok(e) => Ok(e.extract::<Self>(py)?),
            Err(_) => return Err(pyo3::exceptions::PyTypeError::new_err("Python Object passed could not be extracted into a valid trait object.
            \nArgument must be able to be extracted into a rust type implementing the bevy Component and bevy Reflect traits")),
        }
        });

        res
    }

    fn add_transform(&mut self, x: f32, y: f32, z: f32) -> PyResult<Self> {
        //build transform component bundle to handle position
        let trans_bundle = TransformBundle::from_transform(Transform::from_xyz(x, y, z));
        let trans = trans_bundle.local;
        let gtrans = trans_bundle.global;

        self.components.push(Box::new(trans));
        self.components.push(Box::new(gtrans));

        Ok(self.to_owned())
    }

    fn add_velocity(&mut self, x: f32, y: f32, z: f32) -> PyResult<Self> {
        //build velocity component
        let vel_comp = Velocity {
            linvel: Vec3::new(x, y, z),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        };

        self.components.push(Box::new(vel_comp));

        Ok(self.to_owned())
    }

    fn add_geometry(&mut self, geometry: String) -> PyResult<Self> {
        let ci = ColliderInitializer {
            path: PathBuf::from(geometry),
            shape: Shape::Trimesh,
        };

        self.components.push(Box::new(ci));

        Ok(self.to_owned())
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
