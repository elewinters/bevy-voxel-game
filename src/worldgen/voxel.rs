use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;

use bevy::prelude::*;

/* --------------- */
/*      consts     */
/* --------------  */
// #tag enums

const CUBE_VERTEX_POSITIONS: [[f32; 3]; 24] = [
    // front face
    [-0.5, -0.5,  0.5], [0.5, -0.5,  0.5], [0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
    // back face
    [0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5,  0.5, -0.5], [0.5,  0.5, -0.5],
    // right face
    [0.5, -0.5,  0.5], [0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5], [0.5,  0.5,  0.5],
    // left face
    [-0.5, -0.5, -0.5], [-0.5, -0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5],
    // top face
    [-0.5,  0.5,  0.5], [0.5,  0.5,  0.5], [0.5,  0.5, -0.5], [-0.5,  0.5, -0.5],
    // bottom face
    [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5,  0.5], [-0.5, -0.5,  0.5],
];

const CUBE_VERTEX_NORMALS: [[f32; 3]; 24] = [
    // front face
    [0.0,  0.0,  1.0], [0.0,  0.0,  1.0], [0.0,  0.0,  1.0], [0.0,  0.0,  1.0],
    // back face
    [0.0,  0.0, -1.0], [0.0,  0.0, -1.0], [0.0,  0.0, -1.0], [0.0,  0.0, -1.0],
    // right face
    [1.0,  0.0,  0.0], [1.0,  0.0,  0.0], [1.0,  0.0,  0.0], [1.0,  0.0,  0.0],
    // left face
    [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0], [-1.0,  0.0,  0.0],
    // top face
    [0.0,  1.0,  0.0], [0.0,  1.0,  0.0], [0.0,  1.0,  0.0], [0.0,  1.0,  0.0],
    // bottom face
    [0.0, -1.0,  0.0], [0.0, -1.0,  0.0], [0.0, -1.0,  0.0], [0.0, -1.0,  0.0],
];

const CUBE_VERTEX_UVS: [[f32; 2]; 24] = [
    // front face
    [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    // back face
    [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
    // right face
    [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
    // left face
    [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    // top face
    [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    // bottom face
    [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
];

const CUBE_VERTEX_INDICES: [u32; 36] = [
    // front face
    0, 1, 2,  2, 3, 0,
    // back face
    4, 5, 6,  6, 7, 4,
    // right face
    8, 9, 10,  10, 11, 8,
    // left face
    12, 13, 14,  14, 15, 12,
    // top face
    16, 17, 18,  18, 19, 16,
    // bottom face
    20, 21, 22,  22, 23, 20,
];

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
    // worst case scenario
    let max_vertices = 30;

    // new vectors for filtered attributes
    let mut positions = Vec::with_capacity(max_vertices);
    let mut normals = Vec::with_capacity(max_vertices);
    let mut uvs = Vec::with_capacity(max_vertices);
    let mut indices = Vec::with_capacity(max_vertices);

    // map old vertex indices to new ones using a HashMap
    let mut vertex_map = HashMap::with_capacity(16);
    let mut next_vertex_id = 0;

    // process each face that we want in our mesh
    for face in faces {
        let face_start = (face as usize) * 6;
        
        // process 6 vertices for this face
        for old_idx in CUBE_VERTEX_INDICES[face_start..face_start + 6].iter() {
            let old_idx = *old_idx as usize;
            
            // get or insert vertex if it doesn't exist
            let new_idx = *vertex_map.entry(old_idx).or_insert_with(|| {
                positions.push(CUBE_VERTEX_POSITIONS[old_idx]);
                normals.push(CUBE_VERTEX_NORMALS[old_idx]);
                uvs.push(CUBE_VERTEX_UVS[old_idx]);
                
                let id = next_vertex_id;
                next_vertex_id += 1;
                id
            });
            
            indices.push(new_idx);
        }
    }

    // build and return mesh
    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// uses an IVec so that it can be hashed properly
pub fn should_draw_face(face: &VoxelFace, voxel_pos: &IVec3, voxel_positions: &HashSet<IVec3>) -> bool {
    let neighbor_pos = match face {
        VoxelFace::Front => voxel_pos + IVec3::new(0, 0, 1),
        VoxelFace::Back => voxel_pos + IVec3::new(0, 0, -1),
        VoxelFace::Right => voxel_pos + IVec3::new(1, 0, 0),
        VoxelFace::Left => voxel_pos + IVec3::new(-1, 0, 0),
        VoxelFace::Top => voxel_pos + IVec3::new(0, 1, 0),
        VoxelFace::Bottom => return false,
    };
    
    !voxel_positions.contains(&neighbor_pos)
}