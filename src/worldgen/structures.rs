use std::collections::HashSet;
use bevy::prelude::*;

#[derive(Bundle, Clone)]
pub struct StructureBundle(
    Name,
    Transform,
    SceneRoot
);

fn trees(assets: &AssetServer, voxel_positions: &HashSet<IVec3>) -> StructureBundle {
    let first_position = voxel_positions.iter().next().unwrap();
    
    let x = first_position.x as f32;
    let y = first_position.y as f32 + 0.5;
    let z = first_position.z as f32;

    StructureBundle(
        Name::new("test cube"),

        Transform {
            translation: Vec3::new(x, y, z),
            scale: Vec3::new(8.0, 8.0, 8.0),
            ..default()
        },
        SceneRoot(
            assets.load(GltfAssetLabel::Scene(0).from_asset("tree.glb")),
        ),
    )
}

pub fn generate_structures(assets: &AssetServer, voxel_positions: HashSet<IVec3>) -> Vec<StructureBundle> {
    let mut vec = Vec::new();
    vec.push(trees(assets, &voxel_positions));

    vec
}