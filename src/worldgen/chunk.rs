use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;
use noise::*;

use crate::player;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_current_chunk);
        app.add_systems(Startup, startup);

        // spawn_single_chunk responds to SpawnChunkEvents
        app.add_observer(spawn_single_chunk);

        // and all of these respond to CurrentChunkChangedEvents
        app.add_observer(spawn_chunks);
        app.add_observer(despawn_chunks);

        app.insert_resource(GlobalNoise(Perlin::new(512)));
    }
}
/* 
    TODO:
        - make it so that CurrentChunk represents the actual chunk position in world space, 
        instead of having their own positioning system (this is just confusing)
        world_to_chunk could instead just align the world position to the chunk size,
        so that 27.0 becomes 32.0 and 58.0 becomes 64.0

        - make it so that we only have colliders in the current chunk that we're on, this will help performance a lot
        - only render vertices that we see (i made an attempt at this in a previous commit but gave up as the performance gain wasnt big enough)

        - implement greedy meshing maybe????
*/

/* ------------------ */
/*      constants     */
/* ------------------ */
const RENDER_DISTANCE: i32 = 6;
const RENDER_DISTANCE_HALVED: i32 = (RENDER_DISTANCE / 2) as i32;

const CHUNK_SIZE_HORIZONTAL: i32 = 32;
const CHUNK_SIZE_VERTICAL: i32 = 1;

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

// the chunk that the player is currently standing on has changed
// x and z represent the coordinates of the new chunk that we've stepped on
#[derive(Event)]
struct CurrentChunkChangedEvent {
    x: f32,
    z: f32
}

/* ------------------ */
/*      resources     */
/* ------------------ */

#[derive(Resource)]
struct GlobalNoise(Perlin);

/* ------------------- */
/*      components     */
/* ------------------- */
#[derive(Component)]
#[require(Transform, Visibility)]
struct Chunk;

/* ------------------ */
/*      functions     */
/* ------------------ */

// round up to the nearest number divisible by CHUNK_SIZE_HORIZONTAL
fn align_pos_to_chunk(x: f32) -> f32 {
    (CHUNK_SIZE_HORIZONTAL*((x as i32 + (CHUNK_SIZE_HORIZONTAL-1))/CHUNK_SIZE_HORIZONTAL)) as f32
}

/*
    this calculates which chunks should be around the player and what their coordinates should be
    this is a 2D grid of chunks based on the RENDER_DISTANCE (or well, the RENDER_DISTANCE_HALVED)
    we spawn new chunks based on this grid in spawn_chunks, if a chunk doesnt already exist in that position that is
    in case of the render distance being 3 it looks something like this
*/
fn generate_chunk_grid(current_chunk: &CurrentChunkChangedEvent) -> Vec<(f32, f32)> {
    let mut new_chunks: Vec<(f32, f32)> = Vec::new();

    for x in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
        for z in -RENDER_DISTANCE_HALVED..=RENDER_DISTANCE_HALVED {
            let chunk_x = current_chunk.x + (x * CHUNK_SIZE_HORIZONTAL) as f32;
            let chunk_z = current_chunk.z + (z * CHUNK_SIZE_HORIZONTAL) as f32;

            new_chunks.push((chunk_x, chunk_z));
        }
    }

    new_chunks
}

fn generate_noise(perlin_noise: &Perlin, x: f64, y: f64, z: f64) -> f32 {
    let noise_y = perlin_noise.get([x / FLATNESS , y / FLATNESS , z / FLATNESS]) * SPIKINESS;

    noise_y.round() as f32
}

/* ---------------- */
/*      systems     */
/* ---------------- */

// triggers the CurrentChunkChangedEvent so that we actually spawn somewhere
fn startup(mut commands: Commands) {
    commands.trigger(CurrentChunkChangedEvent {
        x: 0.0,
        z: 0.0
    })
}

fn spawn_single_chunk(
    trigger: Trigger<SpawnChunkEvent>,
    perlin_noise: Res<GlobalNoise>,

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

    for x in 0..CHUNK_SIZE_HORIZONTAL {
        for y in 0..CHUNK_SIZE_VERTICAL {
            for z in 0..CHUNK_SIZE_HORIZONTAL {
                // spawn a cube as a child of the chunk
                chunk.with_child((
                    Transform::from_xyz(
                        x as f32, 
                        generate_noise(
                            &perlin_noise.0,

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
    trigger: Trigger<CurrentChunkChangedEvent>,
    mut commands: Commands,

    chunk_query: Query<&Transform, With<Chunk>>,
) {
    let current_chunk = trigger.event();
    /* generate a grid of chunk positions around the player  */
    let new_chunks = generate_chunk_grid(current_chunk);

    // get existing chunks
    let mut existing_chunks = Vec::new();
    for transform in chunk_query {
        existing_chunks.push((transform.translation.x, transform.translation.z));
    }

    // spawn new chunks based on the grid in new_chunks
    for (x, z) in new_chunks {
        // as long as it doesn't exist already
        if existing_chunks.contains(&(x, z)) {
            continue;
        }

        commands.trigger(SpawnChunkEvent {
            chunk_pos_x: x as f32,
            chunk_pos_z: z as f32,
        });
    }
}

fn despawn_chunks(
    trigger: Trigger<CurrentChunkChangedEvent>,
    mut commands: Commands,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    let current_chunk = trigger.event();
    // calculate which chunks are around the player (same as in spawn_chunks)
    let chunks = generate_chunk_grid(current_chunk);

    // check all existing chunks
    for (entity, transform) in chunk_query {
        // if this chunk isn't in the grid, despawn it
        if !chunks.contains(&(transform.translation.x, transform.translation.z)) {
            commands.entity(entity).despawn();
        }
    }
}

fn update_current_chunk(
    mut commands: Commands,
    player_transform: Single<&Transform, With<player::Player>>,

    mut current_chunk: Local<(f32, f32)>
) {
    let player_pos = player_transform.translation;
    
    // snap player position to chunk coordinates
    let chunk_x = align_pos_to_chunk(player_pos.x);
    let chunk_z = align_pos_to_chunk(player_pos.z);

    // only update current_chunk if changed
    if current_chunk.0 != chunk_x || current_chunk.1 != chunk_z {

        current_chunk.0 = chunk_x;
        current_chunk.1 = chunk_z;

        commands.trigger(CurrentChunkChangedEvent {
            x: chunk_x,
            z: chunk_z
        });
        
        println!("player moved to chunk: {chunk_x}, {chunk_z}");
    }
}