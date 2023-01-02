use anyhow::Result;
use bevy::render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, VHACDParameters, Vect};
use std::{fmt, path::Path};
use tobj::load_obj;

pub fn parse_obj_into_trimesh<P>(file_name: P, scale_factor: f32) -> Result<Vec<Collider>>
where
    P: AsRef<Path> + fmt::Debug,
{
    let (model, _) = load_obj(file_name, &tobj::GPU_LOAD_OPTIONS)?;

    let colliders = model
        .iter()
        .map(|mesh| -> (Vec<Vect>, Vec<u32>) { grab_trimesh_data(&mesh.mesh, scale_factor) })
        .map(|g| -> Collider {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, g.0);
            mesh.set_indices(Some(Indices::U32(g.1)));
            Collider::from_bevy_mesh(
                &mesh,
                &ComputedColliderShape::ConvexDecomposition(VHACDParameters::default()),
            )
            .unwrap()
        })
        .collect();

    Ok(colliders)
}

fn grab_trimesh_data(mesh: &tobj::Mesh, scale_factor: f32) -> (Vec<Vect>, Vec<u32>) {
    // let i0s = mesh.indices.iter().step_by(3).copied();
    // let i1s = mesh.indices.iter().skip(1).step_by(3).copied();
    // let i2s = mesh.indices.iter().skip(2).step_by(3).copied();

    // let indices: Vec<[u32; 3]> = i0s
    //     .zip(i1s)
    //     .zip(i2s)
    //     .map(|i| -> [u32; 3] { [i.0 .0, i.0 .1, i.1] })
    //     .collect();

    let x_vec = mesh.positions.iter().step_by(3).copied();
    let y_vec = mesh.positions.iter().skip(1).step_by(3).copied();
    let z_vec = mesh.positions.iter().skip(2).step_by(3).copied();

    let vertices: Vec<Vect> = x_vec
        .zip(y_vec)
        .zip(z_vec)
        .map(|v| -> Vect {
            Vect {
                x: v.0 .0 * scale_factor,
                y: v.0 .1 * scale_factor,
                z: v.1 * scale_factor,
            }
        })
        .collect();

    (vertices, mesh.indices.clone())
}
