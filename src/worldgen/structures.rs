use bevy::prelude::*;

use super::chunk::Chunk;

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(trees);
    }
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn trees(
    event: Trigger<OnAdd, Chunk>,
    query: Query<&Chunk>,

    mut commands: Commands,
    assets: Res<AssetServer>
) {
    let chunk_entity = event.target();
    let chunk = query.get(chunk_entity).unwrap();
    let first_position = chunk.voxel_positions.iter().next().unwrap();
    
    let x = first_position.x as f32;
    let y = first_position.y as f32 + 0.5;
    let z = first_position.z as f32;

    commands.get_entity(chunk_entity).unwrap().with_child((
        Name::new("tree"),

        Transform {
            translation: Vec3::new(x, y, z),
            scale: Vec3::new(8.0, 8.0, 8.0),
            ..default()
        },
        SceneRoot(
            assets.load(GltfAssetLabel::Scene(0).from_asset("tree.glb")),
        ),
    ));
}