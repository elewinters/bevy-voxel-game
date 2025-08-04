use std::collections::HashSet;

use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;
use fastnoise_lite::*;

use crate::player;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_current_chunk);
        app.add_systems(Startup, startup);

        // spawn_single_chunk responds to SpawnChunkEvents
        app.add_observer(spawn_single_chunk);

        // and all of these respond to ChunkChangedEvents
        app.add_observer(spawn_chunks);
        app.add_observer(despawn_chunks);
    }
}
/* 
    TODO:
        - implement face culling (with mesh.indices_mut)
        - add multithreading to chunk generation
*/

/* ------------------ */
/*      constants     */
/* ------------------ */

// chunk generation constants
const RENDER_DISTANCE: i32 = 16;
const CHUNK_GRID_LEN: usize = (RENDER_DISTANCE as usize + 1) * (RENDER_DISTANCE as usize + 1);

const CHUNK_SIZE_HORIZONTAL: i32 = 32;
const CHUNK_SIZE_VERTICAL: i32 = 1;

// noise algorithm constants
const SCALE: f32 = 0.5; // number from 0.0 to 1.0
const SMOOTHNESS: f32 = 75.0;
const HEIGHT_VARIATION: f32 = 50.0;

/* ---------------- */
/*      structs     */
/* ---------------- */

// we use ints here because we want to be able to implement Hash in order to use HashSets
// this also makes loads of other things easier
// but at the end of the day this is just the actual world coordinate data for the chunk, we just cast it to f32 when we want to spawn it
#[derive(Default, Eq, PartialEq, Hash)]
struct ChunkPosition {
    x: i32,
    z: i32
}

impl ChunkPosition {
    fn new(x: i32, z: i32) -> Self {
        Self {x, z}
    }
}

/* --------------- */
/*      events     */
/* --------------- */
#[derive(Event)]
struct SpawnChunkEvent(ChunkPosition);

// the chunk that the player is currently standing on has changed
// ChunkPosition represents the coordinates of the new chunk that we've stepped on
#[derive(Event)]
struct ChunkChangedEvent(ChunkPosition);

/* ------------------ */
/*      resources     */
/* ------------------ */
#[derive(Resource)]
struct PerlinNoise(FastNoiseLite);

/* ------------------- */
/*      components     */
/* ------------------- */
#[derive(Component)]
#[require(Transform, Visibility)]
struct Chunk;

/* ------------------ */
/*      functions     */
/* ------------------ */

// round to the nearest chunk (nearest number divisible by CHUNK_SIZE_HORIZONTAL)
fn align_pos_to_chunk(x: f32) -> i32 {
    let chunk_size = CHUNK_SIZE_HORIZONTAL as f32;
    ((x / chunk_size).floor() * chunk_size) as i32
}

/*
    this calculates a grid of chunk positions around the player based on RENDER_DISTANCE
    we spawn new chunks based on this grid in spawn_chunks, if a chunk doesnt already exist in that position that is
*/
fn generate_chunk_grid(current_chunk: &ChunkPosition) -> HashSet<ChunkPosition> {
    let mut new_chunks: HashSet<ChunkPosition> = HashSet::with_capacity(CHUNK_GRID_LEN);
    let render_half = RENDER_DISTANCE / 2;

    for x in -render_half..=render_half {
        for z in -render_half..=render_half {
            let chunk_x: i32 = current_chunk.x + (x * CHUNK_SIZE_HORIZONTAL);
            let chunk_z: i32 = current_chunk.z + (z * CHUNK_SIZE_HORIZONTAL);

            new_chunks.insert(ChunkPosition::new(chunk_x, chunk_z));
        }
    }

    new_chunks
}

fn generate_noise(perlin_noise: &FastNoiseLite, x: f32, z: f32) -> f32 {
    let noise_y = perlin_noise.get_noise_2d(x / SMOOTHNESS, z / SMOOTHNESS) * HEIGHT_VARIATION;

    noise_y.round()
}

/* ---------------- */
/*      systems     */
/* ---------------- */
fn startup(mut commands: Commands) {
    // setup noise resource
    let mut noise = FastNoiseLite::with_seed(512);
    noise.set_frequency(Some(SCALE));
    noise.set_noise_type(Some(NoiseType::Perlin));

    commands.insert_resource(PerlinNoise(noise));

    // triggers the ChunkChangedEvent so that we actually spawn somewhere
    commands.trigger(ChunkChangedEvent(ChunkPosition::new(0, 0)));
}

fn spawn_single_chunk(
    trigger: Trigger<SpawnChunkEvent>,
    perlin_noise: Res<PerlinNoise>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk_pos = &trigger.event().0;

    // chunk entity, we will append the mesh data later after we generate it
    let mut chunk = commands.spawn((
        Chunk,
        Transform::from_xyz(
            chunk_pos.x as f32,
            0.0,
            chunk_pos.z as f32
        )
    ));

    // the for loop below adds to this mesh to create one big mesh
    let mut final_mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));

    for x in 0..CHUNK_SIZE_HORIZONTAL {
        for _ in 0..CHUNK_SIZE_VERTICAL {
            for z in 0..CHUNK_SIZE_HORIZONTAL {
                // create a mesh
                let mut mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));
                
                // change its position to the one we want
                mesh.translate_by(Vec3::new(
                    x as f32, 
                    generate_noise(
                        &perlin_noise.0,

                        x as f32 + chunk_pos.x as f32,
                        z as f32 + chunk_pos.z as f32
                    ), 
                    z as f32
                ));
                
                // merge the generated mesh with the final mesh
                final_mesh.merge(&mesh).expect("vertex attributes are incompatible with final mesh. this should NEVER happen under normal circumstances");
            }
        }
    }

    // add mesh and collider to the chunk
    chunk.with_child((
        Collider::from_bevy_mesh(&final_mesh, &ComputedColliderShape::default()).expect("incorrect mesh passed to from_bevy_mesh, this will never happen"),

        Mesh3d(meshes.add(final_mesh)),
        MeshMaterial3d(materials.add(Color::from(LAWN_GREEN))),
    ));
}

// spawns new chunks based on the player's position
fn spawn_chunks(
    trigger: Trigger<ChunkChangedEvent>,
    mut commands: Commands,

    chunk_query: Query<&Transform, With<Chunk>>,
) {
    let current_chunk = &trigger.event().0;

    /* generate a grid of chunk positions around the player  */
    let new_chunks = generate_chunk_grid(current_chunk);

    // get existing chunks
    let mut existing_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
    for transform in chunk_query {
        existing_chunks.insert(ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32));
    }

    // spawn new chunks based on the grid in new_chunks
    for pos in new_chunks {
        // as long as it doesn't exist already
        if existing_chunks.contains(&pos) {
            continue;
        }

        commands.trigger(SpawnChunkEvent(pos));
    }
}

fn despawn_chunks(
    trigger: Trigger<ChunkChangedEvent>,
    mut commands: Commands,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    let current_chunk = &trigger.event().0;
    let chunks = generate_chunk_grid(current_chunk);

    // check all existing chunks
    for (entity, transform) in chunk_query {
        // if this chunk isn't in the grid, despawn it
        if !chunks.contains(&ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32)) {
            commands.entity(entity).despawn();
        }
    }
}

fn update_current_chunk(
    mut commands: Commands,
    player_transform: Single<&Transform, With<player::Player>>,

    mut prev_chunk: Local<ChunkPosition>
) {
    let player_pos = player_transform.translation;
    
    // snap player position to chunk coordinates
    let chunk_x = align_pos_to_chunk(player_pos.x);
    let chunk_z = align_pos_to_chunk(player_pos.z);

    // only update current_chunk if changed
    if prev_chunk.x != chunk_x || prev_chunk.z != chunk_z {
        prev_chunk.x = chunk_x;
        prev_chunk.z = chunk_z;

        commands.trigger(ChunkChangedEvent(ChunkPosition::new(chunk_x, chunk_z)));
    }
}