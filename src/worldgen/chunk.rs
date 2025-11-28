use std::sync::Arc;
use std::collections::HashSet;

use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};

use bevy::color::palettes::css::*;
use bevy::prelude::*;

use fastnoise_lite::*;

use crate::player;
use super::{voxel, voxel::VoxelFace};
use super::structures;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // add structures plugin
        app.add_plugins(structures::StructuresPlugin);
        
        app.add_systems(Update, update_current_chunk);
        app.add_systems(Startup, startup);

        // spawns/handles tasks responsible for generating chunks 
        app.add_systems(Update, spawn_chunk_tasks); // responds to the SpawnChunk event
        app.add_systems(Update, handle_chunk_tasks);

        app.add_message::<SpawnChunk>();

        // and all of these respond to ChunkChanged events
        app.add_observer(spawn_chunks_around_player);
        app.add_observer(despawn_chunks);

        // responds to RegenerateChunk event
        app.add_observer(regenerate_chunk);

        app.init_resource::<ChunkQueue>();
    }
}
/* 
    TODO:
        MAYBE:
        - some simple & fast greedy meshing
        - LODs (probably not)
        - speed up chunk generation (optimize compute_chunk_mesh and generate_voxel_mesh)
*/

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

// chunk gen constants
const RENDER_DISTANCE: i32 = 16;
const CHUNK_GRID_LEN: usize = (RENDER_DISTANCE as usize + 1) * (RENDER_DISTANCE as usize + 1);

const CHUNK_SIZE_HORIZONTAL: i32 = 32;
const CHUNK_SIZE_VERTICAL: i32 = 1;
const CHUNK_LEN: usize = (CHUNK_SIZE_HORIZONTAL * CHUNK_SIZE_HORIZONTAL) as usize;

// noise constants
const FREQUENCY: f32 = 0.006; // essentially the scale of the noise function, lower values zoom in while higher values zoom out

const PLAINS_HEIGHT_VARIATION: f32 = 10.0;
const PLAINS_VALLEY_THRESHOLD: f32 = 0.0; // below this value we'll have valleys
const PLAINS_VALLEY_SMOOTHNESS: f32 = 1.5; // how smooth valleys are
const PLAINS_VALLEY_STEP: f32 = 3.0; // how much smoother the valleys should get the lower they are

const HILLS_HEIGHT_VARIATION: f32 = 50.0;
const HILLS_WAVELENGTH: f32 = 5.0;

/* ---------------- */
/*      structs     */
/* ---------------- */
// #tag structs

// we use ints here because we want to be able to implement Hash in order to use HashSets
// this also makes loads of other things easier
// but at the end of the day this is just the actual world coordinate data for the chunk, we just cast it to f32 when we want to spawn it
#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32
}

impl ChunkPosition {
    fn new(x: i32, z: i32) -> Self {
        Self {x, z}
    }
}

pub struct ChunkTaskData {
    transform: Transform,
    voxel_positions: HashSet<IVec3>,
    mesh: Mesh,
}

/* --------------- */
/*      events     */
/* --------------- */
// #tag events

#[derive(Message)]
struct SpawnChunk(ChunkPosition);

// the chunk that the player is currently standing on has changed
// ChunkPosition represents the coordinates of the new chunk that we've stepped on
#[derive(Event)]
pub struct ChunkChanged(pub ChunkPosition);

#[derive(Event)]
pub struct RegenerateChunk(pub Entity);

/* ------------------ */
/*      resources     */
/* ------------------ */
// #tag resources

#[derive(Resource)]
struct GlobalMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct Noise(Arc<FastNoiseLite>);

#[derive(Resource, Default)]
pub struct ChunkQueue(pub Vec<Task<ChunkTaskData>>);

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct Chunk {
    pub voxel_positions: HashSet<IVec3>
}

/* ---------------- */
/*      bundles     */
/* ---------------- */
// #tag bundles

#[derive(Bundle)]
struct ChunkBundle(
    Chunk,
    Name,

    Transform,

    Mesh3d,
    MeshMaterial3d<StandardMaterial>
);

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

// round to the nearest chunk (nearest number divisible by CHUNK_SIZE_HORIZONTAL)
fn align_pos_to_chunk(x: f32) -> i32 {
    let chunk_size = CHUNK_SIZE_HORIZONTAL as f32;
    ((x / chunk_size).floor() * chunk_size) as i32
}

fn terrain_noise(perlin_noise: &FastNoiseLite, x: f32, z: f32) -> f32 {
    let plains = perlin_noise.get_noise_2d(x, z);
    let hills = perlin_noise.get_noise_2d(x / HILLS_WAVELENGTH, z / HILLS_WAVELENGTH) * HILLS_HEIGHT_VARIATION;

    // plains valleys
    let plains = if plains < PLAINS_VALLEY_THRESHOLD {
        // we multiply VALLEY_SMOOTHNESS by (VALLEY_STEP.powf(y) so that the deeper the valley the smoother it is
        plains * PLAINS_HEIGHT_VARIATION / (PLAINS_VALLEY_SMOOTHNESS * (PLAINS_VALLEY_STEP.powf(plains.abs())))
    }
    // normal plains
    else {
        (plains * PLAINS_HEIGHT_VARIATION).powf(1.1)
    };

    (plains + hills).floor()
}

/*
    this calculates a grid of chunk positions around the player based on RENDER_DISTANCE
    we spawn new chunks based on this grid in spawn_chunks, if a chunk doesnt already exist in that position that is
*/
fn chunk_grid(current_chunk: &ChunkPosition) -> HashSet<ChunkPosition> {
    let mut new_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
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

fn chunk_voxel_positions(noise: &FastNoiseLite, chunk_pos: &ChunkPosition) -> HashSet<IVec3> {
    // hashset of voxel positions
    // we use an IVec so that we can hash it
    let mut voxel_positions: HashSet<IVec3> = HashSet::with_capacity(CHUNK_LEN);

    // determine position of each voxel and add to voxel_positions
    for x in 0..CHUNK_SIZE_HORIZONTAL {
        for _ in 0..CHUNK_SIZE_VERTICAL {
            for z in 0..CHUNK_SIZE_HORIZONTAL {
                // determine position
                let voxel_position = Vec3::new(
                    x as f32,
                    terrain_noise(
                        noise,

                        x as f32 + chunk_pos.x as f32,
                        z as f32 + chunk_pos.z as f32
                    ), 
                    z as f32
                );

                // add to voxel_positions
                voxel_positions.insert(voxel_position.as_ivec3());
            }
        }
    }

    voxel_positions
}

fn chunk_mesh(voxel_positions: &HashSet<IVec3>) -> Mesh {
    // chunk mesh, initial value is essentially empty. we add individual voxels to this mesh to generate one big mesh
    let mut chunk_mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));

    // generate mesh based on voxels
    for voxel_pos in voxel_positions {
        let mut faces = VoxelFace::all_faces();

        // only keep the faces that we should draw
        faces.retain(|face| voxel::should_draw_face(face, voxel_pos, voxel_positions));

        // merge mesh
        let mut voxel_mesh = voxel::voxel_mesh(faces);
        voxel_mesh.translate_by(voxel_pos.as_vec3());
        
        chunk_mesh.merge(&voxel_mesh).expect("invalid mesh");
    }

    chunk_mesh
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn startup(
    mut commands: Commands,

    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // setup noise resource
    let mut noise = FastNoiseLite::with_seed(512);
    noise.set_frequency(Some(FREQUENCY));
    noise.set_noise_type(Some(NoiseType::Perlin));
    commands.insert_resource(Noise(Arc::new(noise)));

    // setup global material resource
    let global_material = materials.add(Color::from(LAWN_GREEN));
    commands.insert_resource(GlobalMaterial(global_material));

    // triggers the ChunkChanged event so that we actually spawn somewhere
    commands.trigger(ChunkChanged(ChunkPosition::new(0, 0)));

    // purple test entity
    commands.spawn((
        Name::new("test entity"),

        Transform {
            translation: Vec3::new(3.0, -0.5, 3.0),
            scale: Vec3::new(8.0, 8.0, 8.0),
            ..default()
        },
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("tree.glb")),
        ),
    ));
}

// reacts to the SpawnChunk event and spawns a Task that computes the specified chunk with the given chunk positions
// we allow this Task to run over several frames, when that task is complete we handle it in handle_chunk_tasks, which actually spawns the chunk
fn spawn_chunk_tasks(
    mut reader: MessageReader<SpawnChunk>,
    mut queue: ResMut<ChunkQueue>,
    noise: Res<Noise>,
) {
    for SpawnChunk(chunk_pos) in reader.read() {
        let noise = Arc::clone(&noise.0);
        let chunk_pos = Arc::new(chunk_pos.clone());
        
        // spawn task that computes the specified chunks, returning their positions and meshes
        let task = AsyncComputeTaskPool::get().spawn(async move {
            let voxel_positions = chunk_voxel_positions(&noise, &chunk_pos);
            let mesh = chunk_mesh(&voxel_positions);

            ChunkTaskData {
                transform: Transform::from_xyz(
                    chunk_pos.x as f32,
                    0.0,
                    chunk_pos.z as f32
                ),

                voxel_positions,
                mesh
            }
        });

        // push task to task queue
        queue.0.push(task);
    }
}

// we handle chunk tasks here, checking if a given task is finished and then spawning the chunk
fn handle_chunk_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkQueue>,

    mut meshes: ResMut<Assets<Mesh>>,
    global_material: Res<GlobalMaterial>,
) {
    // remove tasks from the queue that have finished and have spawned successfully
    queue.0.retain_mut(|task| match block_on(future::poll_once(task)) {
        Some(chunk_data) => {
            commands.spawn(ChunkBundle(
                Chunk {
                    voxel_positions: chunk_data.voxel_positions.clone()
                },
                Name::new("chunk"),

                chunk_data.transform,

                Mesh3d(meshes.add(chunk_data.mesh.clone())),
                MeshMaterial3d(global_material.0.clone()),
            ));

            // remove from the queue, as the task has finished 
            false
        }
        None => true, // keep it in the queue, as the task is still processing
    });
}

// spawns new chunks based on the player's position, runs when the ChunkChanged event is triggered
// triggers the SpawnChunk event
fn spawn_chunks_around_player(
    event: On<ChunkChanged>,
    mut writer: MessageWriter<SpawnChunk>,

    chunk_query: Query<&Transform, With<Chunk>>,
) {
    /* generate a grid of chunk positions around the player  */
    let mut new_chunks = chunk_grid(&event.0);

    // get existing chunks
    let mut existing_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
    for transform in chunk_query {
        existing_chunks.insert(ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32));
    }

    // only keep the positions that dont exist, so that we dont spawn new chunks in a place where a chunk already exists
    new_chunks.retain(|pos| !existing_chunks.contains(pos));

    // spawn chunks based on chunk positions hashset
    for chunk in new_chunks {
        writer.write(SpawnChunk(chunk));
    }
}

// despawns chunks that aren't in the chunk grid
// runs when the ChunkChanged event triggers 
fn despawn_chunks(
    event: On<ChunkChanged>,
    mut commands: Commands,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    let chunk_grid = chunk_grid(&event.0);

    // iterate over all exisiting chunks
    for (entity, transform) in chunk_query {
        // check chunk grid, if this chunk isn't in the grid, despawn it
        if !chunk_grid.contains(&ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32)) {
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

        commands.trigger(ChunkChanged(ChunkPosition::new(chunk_x, chunk_z)));
    }
}

fn regenerate_chunk(
    event: On<RegenerateChunk>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    global_material: Res<GlobalMaterial>,

    chunk_query: Query<(&Chunk, &Transform)>,
) {
    let entity = event.0;

    // get chunk component and chunk transform through entity
    let (chunk, chunk_transform) = match chunk_query.get(entity) {
        Ok(x) => x,
        _ => return
    };

    // despawn chunk
    commands.entity(entity).despawn();

    // generate new mesh based on new voxel_positions
    let mesh = chunk_mesh(&chunk.voxel_positions);

    // spawn new chunk
    commands.spawn(ChunkBundle(
        chunk.clone(),
        Name::new("chunk"),
        
        *chunk_transform, // this performs a copy

        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(global_material.0.clone()),
    ));
}