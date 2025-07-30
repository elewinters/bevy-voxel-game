use std::collections::HashSet;

use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;
use noise::*;

use crate::player;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_current_chunk, 
            spawn_chunks,
            despawn_chunks
        ));

        // spawn_single_chunk responds to SpawnChunkEvents
        app.add_observer(spawn_single_chunk);

        app.init_resource::<CurrentChunk>();
    }
}

/* ------------------ */
/*      constants     */
/* ------------------ */
const RENDER_DISTANCE: u32 = 6;
const RENDER_DISTANCE_HALVED: i32 = (RENDER_DISTANCE / 2) as i32;

const CHUNK_SIZE_X: u32 = 32;
const CHUNK_SIZE_Y: u32 = 1;
const CHUNK_SIZE_Z: u32 = 32;

const FLATNESS: f64 = 60.0;
const SPIKINESS: f64 = 20.0;

/* --------------- */
/*      events     */
/* --------------- */
#[derive(Event)]
struct SpawnChunkEvent {
    chunk_pos_x: f32,
    chunk_pos_z: f32
}

/* ------------------ */
/*      resources     */
/* ------------------ */

// x and z represent chunk coordinates, which are integers that represent the position of the chunk in the grid
#[derive(Resource, Default)]
struct CurrentChunk {
    x: i32,
    z: i32
}

/* ------------------- */
/*      components     */
/* ------------------- */
#[derive(Component)]
#[require(Transform, Visibility)]
struct Chunk;

/* ------------------ */
/*      functions     */
/* ------------------ */
fn generate_noise(x: f64, y: f64, z: f64) -> f32 {
    let noise = Perlin::new(512);
    let noise_y = noise.get([x / FLATNESS , y / FLATNESS , z / FLATNESS]) * SPIKINESS;

    noise_y.round() as f32
}

/* ---------------- */
/*      systems     */
/* ---------------- */
fn spawn_single_chunk(
    trigger: Trigger<SpawnChunkEvent>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let event = trigger.event();
    let mut chunk = commands.spawn((
        Chunk,
        Transform::from_xyz(
            event.chunk_pos_x,
            0.0,
            event.chunk_pos_z
        )
    ));

    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::from(LAWN_GREEN));

    for x in 0..CHUNK_SIZE_X {
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                // spawn a cube as a child of the chunk
                chunk.with_child((
                    Transform::from_xyz(
                        x as f32, 
                        generate_noise(
                            x as f64 + event.chunk_pos_x as f64,
                            y as f64, 
                            z as f64 + event.chunk_pos_z as f64
                        ), 
                        z as f32
                    ),
                    Collider::cuboid(0.5, 0.5, 0.5),

                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                ));
            }
        }
    }
}

// spawns new chunks based on the player's position
fn spawn_chunks(
    mut commands: Commands,

    current_chunk: Res<CurrentChunk>,
    chunk_query: Query<&Transform, With<Chunk>>,
) {
    /*
        calculate chunk coordinates that should exist
        desired_chunks is a 2D grid of chunks based on the RENDER_DISTANCE
        in case of the render distance being 3 it looks something like this
    
        [-1,-1] [0,-1] [1,-1]
        [-1, 0] [0, 0] [1, 0]
        [-1, 1] [0, 1] [1, 1]
    */
    let mut desired_chunks = HashSet::new();
    for x in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
        for z in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
            let chunk_x = current_chunk.x + x;
            let chunk_z = current_chunk.z + z;

            desired_chunks.insert((chunk_x, chunk_z));
        }
    }

    // track existing chunks
    let mut existing_chunks = HashSet::new();
    for transform in chunk_query {
        // convert from world coordinates to chunk coordinates
        // floor() is for rounding up (important for negative coordinates)
        let chunk_x = (transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let chunk_z = (transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;

        existing_chunks.insert((chunk_x, chunk_z));
    }

    // spawn new chunks that don't exist yet but should exist
    for (x, z) in desired_chunks {
        if existing_chunks.contains(&(x, z)) {
            continue;
        }

        commands.trigger(SpawnChunkEvent {
            // convert back to world coordinates
            chunk_pos_x: (x * CHUNK_SIZE_X as i32) as f32,
            chunk_pos_z: (z * CHUNK_SIZE_Z as i32) as f32,
        });
    }
}

fn despawn_chunks(
    mut commands: Commands,
    current_chunk: Res<CurrentChunk>,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    // calculate which chunks should exist (same as in spawn_chunks)
    let mut desired_chunks = HashSet::new();
    for x in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
        for z in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
            let chunk_x = current_chunk.x + x;
            let chunk_z = current_chunk.z + z;
            desired_chunks.insert((chunk_x, chunk_z));
        }
    }

    // check all existing chunks
    for (entity, transform) in chunk_query {
        // convert chunk position to chunk coordinates
        let chunk_x = (transform.translation.x / CHUNK_SIZE_X as f32).floor() as i32;
        let chunk_z = (transform.translation.z / CHUNK_SIZE_Z as f32).floor() as i32;

        // if this chunk shouldn't exist anymore, despawn it
        if !desired_chunks.contains(&(chunk_x, chunk_z)) {
            commands.entity(entity).despawn();
        }
    }
}

fn update_current_chunk(
    mut current_chunk: ResMut<CurrentChunk>,
    player_transform: Single<&Transform, With<player::Player>>,
) {
    let player_pos = player_transform.translation;
    
    // convert player position to chunk coordinates
    let chunk_x = (player_pos.x / CHUNK_SIZE_X as f32).floor() as i32;
    let chunk_z = (player_pos.z / CHUNK_SIZE_Z as f32).floor() as i32;

    // only update current_chunk if changed
    if current_chunk.x != chunk_x || current_chunk.z != chunk_z {
        current_chunk.x = chunk_x;
        current_chunk.z = chunk_z;
        
        println!("player moved to chunk: {}, {}", chunk_x, chunk_z);
    }
}