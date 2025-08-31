use bevy::prelude::*;
use fastnoise_lite::*;

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
    query: Query<(&Chunk, &Transform)>,

    mut commands: Commands,
    assets: Res<AssetServer>
) {
    // get chunk
    let chunk_entity = event.target();
    let (chunk, chunk_transform) = query.get(chunk_entity).unwrap();
    
    // get noise
    let mut noise = FastNoiseLite::with_seed(512);
    noise.set_frequency(Some(0.5));
    noise.set_noise_type(Some(NoiseType::Perlin));

    for local_pos in &chunk.voxel_positions {
        // position of the voxel in global world space
        let global_pos = local_pos.as_vec3() - chunk_transform.translation;
        // noise value, if this is above 0.75 we spawn a tree
        let value = noise.get_noise_2d(global_pos.x, global_pos.z);

        if value < 0.75 {
            continue;
        }

        // if value is > 0.75
        commands.get_entity(chunk_entity).unwrap().with_child((
            Name::new("tree"),
            Transform {
                translation: Vec3::new(local_pos.x as f32, local_pos.y as f32 + 0.5, local_pos.z as f32),
                scale: Vec3::new(8.0, 8.0, 8.0),
                ..default()
            },

            SceneRoot(
                assets.load(GltfAssetLabel::Scene(0).from_asset("tree.glb")),
            ),
        ));
    }
}