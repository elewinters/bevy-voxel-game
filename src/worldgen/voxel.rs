use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{VertexAttributeValues, Indices};

use bevy::prelude::*;

/* -------------- */
/*      enums     */
/* -------------  */
// #tag enums

pub enum VoxelFace {
    Front = 0,
    Back,
    Right,
    Left,
    Top,
    Bottom
}

impl VoxelFace {
    pub fn all_faces() -> Vec<VoxelFace> {
        vec![
            VoxelFace::Front,
            VoxelFace::Back,
            VoxelFace::Right,
            VoxelFace::Left,
            VoxelFace::Top,
            VoxelFace::Bottom
        ]
    }
}

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

pub fn voxel_mesh(faces: Vec<VoxelFace>) -> Mesh {
    let source_mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));
    
    // extract attributes from source mesh
    let (positions, normals, uvs, indices) = {
        let pos = match source_mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(vec) => vec,
            _ => panic!("positions are not in 32x3 format")
        };
        let norm = match source_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
            VertexAttributeValues::Float32x3(vec) => vec,
            _ => panic!("normals are not in 32x3 format")
        };
        let uv = match source_mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
            VertexAttributeValues::Float32x2(vec) => vec,
            _ => panic!("UVs are not in 32x2 format")
        };
        let idx = match source_mesh.indices().unwrap() {
            Indices::U32(vec) => vec,
            _ => panic!("expected U32 indices for mesh"),
        };
        (pos, norm, uv, idx)
    };

    // worst case scenario
    let max_vertices = 30;

    // new vectors for filtered attributes
    let mut new_positions = Vec::with_capacity(max_vertices);
    let mut new_normals = Vec::with_capacity(max_vertices);
    let mut new_uvs = Vec::with_capacity(max_vertices);
    let mut new_indices = Vec::with_capacity(max_vertices);

    // map old vertex indices to new ones using a HashMap
    let mut vertex_map = HashMap::with_capacity(16);
    let mut next_vertex_id = 0;

    // process each face that we want in our mesh
    for face in faces {
        let face_start = (face as usize) * 6;
        
        // process 6 vertices for this face
        for &old_idx in &indices[face_start..face_start + 6] {
            let old_idx = old_idx as usize;
            
            // get or insert vertex if it doesn't exist
            let new_idx = *vertex_map.entry(old_idx).or_insert_with(|| {
                new_positions.push(positions[old_idx]);
                new_normals.push(normals[old_idx]);
                new_uvs.push(uvs[old_idx]);
                
                let id = next_vertex_id;
                next_vertex_id += 1;
                id
            });
            
            new_indices.push(new_idx);
        }
    }

    // build and return mesh
    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, new_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs)
    .with_inserted_indices(Indices::U32(new_indices))
}

// uses an IVec so that it can be hashed properly
pub fn should_draw_face(face: &VoxelFace, voxel_pos: &IVec3, voxel_positions: &HashSet<IVec3>) -> bool {
    let neighbor_pos = match face {
        VoxelFace::Front => voxel_pos + IVec3::new(0, 0, 1),
        VoxelFace::Back => voxel_pos + IVec3::new(0, 0, -1),
        VoxelFace::Right => voxel_pos + IVec3::new(1, 0, 0),
        VoxelFace::Left => voxel_pos + IVec3::new(-1, 0, 0),
        VoxelFace::Top => voxel_pos + IVec3::new(0, 1, 0),
        VoxelFace::Bottom => voxel_pos + IVec3::new(0, -1, 0),
    };
    
    !voxel_positions.contains(&neighbor_pos)
}