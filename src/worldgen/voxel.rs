use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;

use bevy::prelude::*;

/* --------------- */
/*      macros     */
/* --------------  */
// #tag macros

// repeat all 4 UV values for all 6 faces of a cube
// saves us from having to copy and paste the same 4 arrays 6 times
macro_rules! repeat_uvs {
    ($uv0:expr, $uv1:expr, $uv2:expr, $uv3:expr) => {
        [
            $uv0, $uv1, $uv2, $uv3,
            $uv0, $uv1, $uv2, $uv3,
            $uv0, $uv1, $uv2, $uv3,
            $uv0, $uv1, $uv2, $uv3,
            $uv0, $uv1, $uv2, $uv3,
            $uv0, $uv1, $uv2, $uv3,
        ]
    };
}

/* --------------- */
/*      consts     */
/* --------------  */
// #tag enums

/*
    (-0.5,0.5,-0.5)_________(0.5,0.5,-0.5)
           /|              /|
          / |             / |
(-0.5,0.5,0.5)_________(0.5,0.5,0.5)
        |  |            |  |
        |  |(-0.5,-0.5,-0.5)|_(0.5,-0.5,-0.5)
        | /             | /
        |/              |/
(-0.5,-0.5,0.5)_______(0.5,-0.5,0.5)
    
    closer to camera, higher Z
    further from camera, lower Z
*/
const CUBE_VERTEX_POSITIONS: [[f32; 3]; 24] = [
    // front face
    //    index 0            index 1           index 2            index 3
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

/*
    normals tell the gpu which direction a face is "facing" so that lighting can be calculated properly
    this is very simple for a cube mesh

        TOP FACE
        Normal: [0, 1, 0]
             ↑ (pointing UP)
    ___________
   |           |
   |   CUBE    |
   |___________|
   
   FRONT FACE
   Normal: [0, 0, 1]
        (pointing OUT toward camera →)
*/
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

/*
    the UV coordinate system in bevy is different from the rest of the engine
    top left is the origin instead of the bottom left
    this is consistent with vulkan/dx12 but NOT opengl

    (0,0) -------- (1,0)
      |              |
      |   TEXTURE    |
      |              |
    (0,1) -------- (1,1)
*/

const _CUBE_VERTEX_UVS_FULL: [[f32; 2]; 24] = repeat_uvs!(
    [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]
);

const CUBE_VERTEX_UVS_GRASS: [[f32; 2]; 24] = repeat_uvs!(
    [0.0, 0.5], [0.5, 0.5], [0.5, 0.0], [0.0, 0.0]
);

const CUBE_VERTEX_UVS_DIRT: [[f32; 2]; 24] = repeat_uvs!(
    [0.5, 0.5], [1.0, 0.5], [1.0, 0.0], [0.5, 0.0]
);

/*
    front face has 4 vertices (positions defined above in CUBE_VERTEX_POSITIONS):
    3 --------- 2
    |           |
    |           |
    0 --------- 1

    we split this into triangles
           2
          /|
         / |
        /  |
       /   |
      /    |
     /     |
    0 ---- 1

    3 ---- 2
    |      /
    |     /
    |    /
    |   /
    |  /
    | /
    0

    we use these triangles to form a square (imagine these 2 triangles connecting with each other)

    3 ---- 2
    |   /  |
    |  /   |
    | /    |
    0 ---- 1

*/
const CUBE_VERTEX_INDICES: [u32; 36] = [
    // front face
    0, 1, 2,  
    3, 0, 2,
    // back face
    4, 5, 6,  
    6, 7, 4,
    // right face
    8, 9, 10,  
    10, 11, 8,
    // left face
    12, 13, 14,  
    14, 15, 12,
    // top face
    16, 17, 18,  
    18, 19, 16,
    // bottom face
    20, 21, 22,  
    22, 23, 20,
];

/* -------------- */
/*      enums     */
/* -------------  */
// #tag enums

pub enum Face {
    Front = 0,
    Back,
    Right,
    Left,
    Top,
    Bottom
}

impl Face {
    pub fn all() -> Vec<Face> {
        vec![
            Face::Front,
            Face::Back,
            Face::Right,
            Face::Left,
            Face::Top,
            Face::Bottom
        ]
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Texture {
    Grass,
    Dirt,
}

impl Texture {
    fn uv_coords(&self) -> &'static [[f32; 2]] {
        match self {
            Texture::Grass => &CUBE_VERTEX_UVS_GRASS,
            Texture::Dirt => &CUBE_VERTEX_UVS_DIRT
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct VoxelData {
    pub position: IVec3,
    pub texture: Texture
}

impl VoxelData {
    pub fn new(position: IVec3, texture: Texture) -> VoxelData {
        Self {
            position,
            texture
        }
    }
}

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

pub fn voxel_mesh(faces: Vec<Face>, texture: &Texture) -> Mesh {
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
                uvs.push(texture.uv_coords()[old_idx]);
                
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
pub fn should_draw_face(face: &Face, voxel_pos: &IVec3, voxels: &Vec<VoxelData>) -> bool {
    let neighbor_pos = match face {
        Face::Front => voxel_pos + IVec3::new(0, 0, 1),
        Face::Back => voxel_pos + IVec3::new(0, 0, -1),
        Face::Right => voxel_pos + IVec3::new(1, 0, 0),
        Face::Left => voxel_pos + IVec3::new(-1, 0, 0),
        Face::Top => voxel_pos + IVec3::new(0, 1, 0),
        Face::Bottom => return false,
    };
    
    !voxels.iter().any(|voxel| voxel.position == neighbor_pos)
}