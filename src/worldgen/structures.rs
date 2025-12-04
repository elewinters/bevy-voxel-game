use std::sync::LazyLock;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use fastnoise_lite::*;

use super::chunk::Chunk;
use super::voxel::Texture;

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(structures);
    }
}

#[derive(PartialEq)]
struct Structure {
    model: &'static str,

    translation_offset: Vec3,
    scale: Vec3,

    // determines which blocks the structure can spawn on
    // the float represents the threshold, lower values mean it's more likely to spawn while higher values mean it's less likely to spawn
    // this allows us to have less grass on dirt blocks or only spawn mushrooms on dirt blocks for example
    spawn_on: HashMap<Texture, f32>
}

static STRUCTURE_DEFINITIONS: LazyLock<[Structure; 8]> = LazyLock::new(|| {[
    // tree
    Structure {
        model: "models/tree_textured.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.75),
            (Texture::Dirt, 0.75),
        ]),

        translation_offset: Vec3::ZERO,
        scale: Vec3::new(0.5, 0.5, 0.5),
    },
    // bush
    Structure {
        model: "models/bush.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.85),
            (Texture::Dirt, 0.9),
        ]),

        translation_offset: Vec3::new(0.0, 1.25, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // grass 1
    Structure {
        model: "models/grass1.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.5),
            (Texture::Dirt, 0.7),
        ]),

        translation_offset: Vec3::new(0.0, 1.05, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // grass 2
    Structure {
        model: "models/grass2.glb",
        spawn_on:HashMap::from([
            (Texture::Grass, 0.5),
            (Texture::Dirt, 0.7),
        ]),

        translation_offset: Vec3::new(0.0, 1.0, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // mushroom
    Structure {
        model: "models/mushroom.glb",
        spawn_on: HashMap::from([
            (Texture::Dirt, 0.85),
        ]),

        translation_offset: Vec3::new(0.0, 1.0, 0.0),
        scale: Vec3::new(2.5, 2.5, 2.5)
    },
    // rose
    Structure {
        model: "models/rose.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.8),
            (Texture::Dirt, 0.9),
        ]),

        translation_offset: Vec3::new(0.0, 0.9, 0.0),
        scale: Vec3::new(1.5, 1.5, 1.5)
    },
    // rock
    Structure {
        model: "models/rock.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.9),
            (Texture::Dirt, 0.8),
        ]),

        translation_offset: Vec3::new(0.0, 0.6, 0.0),
        scale: Vec3::new(0.3, 0.3, 0.3)
    },
    // stick
    Structure {
        model: "models/stick.glb",
        spawn_on: HashMap::from([
            (Texture::Grass, 0.9),
            (Texture::Dirt, 0.8),
        ]),

        translation_offset: Vec3::new(0.0, 0.6, 0.0),
        scale: Vec3::new(0.35, 0.35, 0.35)
    }
]});

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

    // store the positions of blocks that we have already spawned a structure so we don't spawn structures in the same spot multiple times
    let mut taken_blocks = HashSet::new();

    for (i, structure) in STRUCTURE_DEFINITIONS.iter().enumerate() {
        // get noise
        let mut noise = FastNoiseLite::with_seed(i as i32);
        noise.set_frequency(Some(0.5));
        noise.set_noise_type(Some(NoiseType::Perlin));

        for voxel_data in &chunk.voxels {
            let local_pos = voxel_data.position;
            // position of the voxel in global world space
            let global_pos = local_pos.as_vec3() - chunk_transform.translation;
            // noise value, if this is above the threshold we spawn the structure
            let value = noise.get_noise_2d(global_pos.x, global_pos.z);

            let threshold = match structure.spawn_on.get(&voxel_data.texture) {
                Some(x) => x,
                None => continue
            };

            if value < *threshold {
                continue;
            }

            if !taken_blocks.insert(local_pos) {
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