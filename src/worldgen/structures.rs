use bevy::prelude::*;
use fastnoise_lite::*;

use super::chunk::Chunk;

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(structures);
    }
}

struct Structure {
    model: &'static str,

    noise_seed: i32,
    threshold: f32,
    translation_offset: Vec3,
    scale: Vec3
}

const STRUCTURE_DEFINITIONS: [Structure; 4] = [
    // tree
    Structure {
        model: "models/tree_textured.glb",

        noise_seed: 1,
        threshold: 0.75,

        translation_offset: Vec3::ZERO,
        scale: Vec3::new(0.5, 0.5, 0.5),
    },
    // bush
    Structure {
        model: "models/bush.glb",

        noise_seed: 2,
        threshold: 0.85,

        translation_offset: Vec3::new(0.0, 1.25, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // grass 1
    Structure {
        model: "models/grass1.glb",

        noise_seed: 3,
        threshold: 0.5,

        translation_offset: Vec3::new(0.0, 1.05, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // grass 2
    Structure {
        model: "models/grass2.glb",

        noise_seed: 4,
        threshold: 0.5,

        translation_offset: Vec3::new(0.0, 1.0, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    }
];

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn structures(
    event: On<Add, Chunk>,
    query: Query<(&Chunk, &Transform)>,

    mut commands: Commands,
    assets: Res<AssetServer>
) {
    // get chunk
    let chunk_entity = event.entity;
    let (chunk, chunk_transform) = query.get(chunk_entity).unwrap();

    for structure in STRUCTURE_DEFINITIONS.iter() {
        // get noise
        let mut noise = FastNoiseLite::with_seed(structure.noise_seed);
        noise.set_frequency(Some(0.5));
        noise.set_noise_type(Some(NoiseType::Perlin));

        for chunk_data in &chunk.voxels {
            let local_pos = chunk_data.position;
            // position of the voxel in global world space
            let global_pos = local_pos.as_vec3() - chunk_transform.translation;
            // noise value, if this is above the threshold we spawn the structure
            let value = noise.get_noise_2d(global_pos.x, global_pos.z);

            if value < structure.threshold {
                continue;
            }

            // if value is > threshold
            commands.get_entity(chunk_entity).unwrap().with_child((
                Name::new(structure.model),
                Transform {
                    translation: Vec3::new(local_pos.x as f32, local_pos.y as f32, local_pos.z as f32) + structure.translation_offset,
                    scale: structure.scale,
                    ..default()
                },

                SceneRoot(
                    assets.load(GltfAssetLabel::Scene(0).from_asset(structure.model)),
                ),
            ));
        }
    }
}