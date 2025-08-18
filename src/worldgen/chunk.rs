use std::sync::Arc;
use std::collections::{HashSet, HashMap};

use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};

use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::mesh::Indices;

use bevy::color::palettes::css::*;
use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use fastnoise_lite::*;

use crate::player;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_current_chunk);
        app.add_systems(Startup, startup);

        // spawns/handles tasks responsible for generating chunks 
        app.add_observer(spawn_chunks_tasks); // responds to SpawnChunksEvent
        app.add_systems(Update, handle_chunks_tasks);

        // and all of these respond to ChunkChangedEvents
        app.add_observer(spawn_chunks_around_player);
        app.add_observer(despawn_chunks);

        // responds to RegenerateChunkEvent
        app.add_observer(regenerate_chunk);

        app.init_resource::<ChunkQueue>();
    }
}
/* 
    TODO:
        - fix face culling (later though)

        - speed up chunk generation by parallelizing the for loop that checks should_draw_face in compute_chunk with par_splat_map
        this will require the use of a Vec instead of a HashSet, so this will be a part of a larger rework of compute_chunk
        that makes it so that instead of passing in the entire voxel positions HashSet to should_draw_face, and then that checking if the position is inside the hashset
        instead we will just send the 6 nearest positions to should_draw_face, which will not only speed things up but allow us to use regular Vecs instead of IVecs

        MAYBE:
        - frustum culling
        - occlusion culling
        - some simple & fast greedy meshing
        - LODs (probably not)
        - data compression with funky bitwise stuff
        - speed up chunk generation
*/

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

// chunk generation constants
const RENDER_DISTANCE: i32 = 16;
const CHUNK_GRID_LEN: usize = (RENDER_DISTANCE as usize + 1) * (RENDER_DISTANCE as usize + 1);

const CHUNK_SIZE_HORIZONTAL: i32 = 32;
const CHUNK_SIZE_VERTICAL: i32 = 1;
const CHUNK_LEN: usize = (CHUNK_SIZE_HORIZONTAL * CHUNK_SIZE_HORIZONTAL) as usize;

// noise algorithm constants
const SCALE: f32 = 0.5; // number from 0.0 to 1.0
const SMOOTHNESS: f32 = 75.0;
const HEIGHT_VARIATION: f32 = 50.0;

/* -------------- */
/*      enums     */
/* -------------  */
// #tag enums

enum VoxelFace {
    Front = 0,
    Back,
    Right,
    Left,
    Top,
    Bottom
}

/* ---------------- */
/*      structs     */
/* ---------------- */
// #tag structs

// we use ints here because we want to be able to implement Hash in order to use HashSets
// this also makes loads of other things easier
// but at the end of the day this is just the actual world coordinate data for the chunk, we just cast it to f32 when we want to spawn it
#[derive(Default, Clone, Eq, PartialEq, Hash)]
struct ChunkPosition {
    x: i32,
    z: i32
}

impl ChunkPosition {
    fn new(x: i32, z: i32) -> Self {
        Self {x, z}
    }
}

#[derive(Default)]
struct ChunkTaskData {
    voxel_positions: Vec<HashSet<IVec3>>,

    transforms: Vec<Transform>,
    meshes: Vec<Mesh>,
    colliders: Vec<Collider>,
}

/* --------------- */
/*      events     */
/* --------------- */
// #tag events

#[derive(Event)]
struct SpawnChunksEvent(HashSet<ChunkPosition>);

// the chunk that the player is currently standing on has changed
// ChunkPosition represents the coordinates of the new chunk that we've stepped on
#[derive(Event)]
struct ChunkChangedEvent(ChunkPosition);

#[derive(Event)]
pub struct RegenerateChunkEvent(pub Entity);

/* ------------------ */
/*      resources     */
/* ------------------ */
// #tag resources

#[derive(Resource)]
struct Noise(Arc<FastNoiseLite>);

#[derive(Resource, Default)]
struct ChunkQueue(Vec<Task<ChunkTaskData>>);

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
    Transform,
    Collider,

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

    noise_y.floor()
}

// generates a voxel mesh based on the faces specified in faces_to_keep
fn generate_voxel_mesh(faces_to_keep: Vec<VoxelFace>) -> Mesh {
    let mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));

    // get all attributes of Cuboid mesh
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(x) => x,
        _ => panic!("positions are not in 32x3 format")
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
        VertexAttributeValues::Float32x3(x) => x,
        _ => panic!("normals are not in 32x3 format")
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(x) => x,
        _ => panic!("UVs are not in 32x2 format")
    };
    
    let indices = match mesh.indices().unwrap() {
        Indices::U32(vec) => vec,
        _ => panic!("expected U32 indices for mesh"),
    };

    // new vectors for filtered attributes
    let mut new_positions = Vec::new();
    let mut new_normals = Vec::new();
    let mut new_uvs = Vec::new();
    let mut new_indices = Vec::new();

    // map old vertex indices to new ones using a HashMap
    let mut vertex_map = HashMap::new();
    let mut next_vertex_id = 0;

    // process each face we want to keep
    for face in faces_to_keep {
        let face_idx = face as usize * 6;
        let face_vertices = &indices[face_idx..face_idx + 6];

        // for each vertex in this face
        for &old_idx in face_vertices {
            let old_idx = old_idx as usize;
            
            // get or create new index for this vertex
            let new_idx = *vertex_map.entry(old_idx).or_insert_with(|| {
                // first time seeing this vertex - add its data
                new_positions.push([
                    positions[old_idx][0],
                    positions[old_idx][1],
                    positions[old_idx][2]
                ]);
                new_normals.push([
                    normals[old_idx][0],
                    normals[old_idx][1],
                    normals[old_idx][2]
                ]);
                new_uvs.push([
                    uvs[old_idx][0],
                    uvs[old_idx][1]
                ]);

                let id = next_vertex_id;
                next_vertex_id += 1;
                id
            });

            // add the new index
            new_indices.push(new_idx);
        }
    }

    // create new mesh with filtered data
    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, new_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs)
    .with_inserted_indices(Indices::U32(new_indices))
}

// uses an IVec so that it can be hashed properly
// this function doesn't work fully properly because of the CHUNK_VERTICAL_SIZE being 1
// tho im not sure why it cant work with it just being 1
// but oh well, ill fix this later i think
fn should_draw_face(face: VoxelFace, voxel_pos: &IVec3, voxel_positions: &HashSet<IVec3>) -> bool {
    let neighbor_pos = match face {
        VoxelFace::Front => voxel_pos + IVec3::new(0, 0, 1),
        VoxelFace::Back => voxel_pos + IVec3::new(0, 0, -1),
        VoxelFace::Right => voxel_pos + IVec3::new(1, 0, 0),
        VoxelFace::Left => voxel_pos + IVec3::new(-1, 0, 0),
        VoxelFace::Top => voxel_pos + IVec3::new(0, 1, 0),
        VoxelFace::Bottom => voxel_pos + IVec3::new(0, -1, 0),
    };
    
    !voxel_positions.contains(&neighbor_pos)
}

fn compute_chunk_voxel_positions(noise: &FastNoiseLite, chunk_pos: &ChunkPosition) -> HashSet<IVec3> {
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
                    generate_noise(
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

fn compute_chunk_mesh(voxel_positions: &HashSet<IVec3>) -> (Mesh, Collider) {
    // chunk mesh, initial value is essentially empty. we add individual voxels to this mesh to generate one big mesh
    let mut chunk_mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));

    // generate mesh based on voxels
    for voxel_pos in voxel_positions {
        // determine which faces to keep for this mesh
        let mut faces_to_keep = Vec::with_capacity(6);
        
        // check each face
        if should_draw_face(VoxelFace::Front, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Front);
        }
        if should_draw_face(VoxelFace::Back, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Back);
        }
        if should_draw_face(VoxelFace::Right, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Right);
        }
        if should_draw_face(VoxelFace::Left, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Left);
        }
        if should_draw_face(VoxelFace::Top, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Top);
        }
        if should_draw_face(VoxelFace::Bottom, voxel_pos, voxel_positions) {
            faces_to_keep.push(VoxelFace::Bottom);
        }

        // merge mesh
        let mut voxel_mesh = generate_voxel_mesh(faces_to_keep);
        voxel_mesh.translate_by(voxel_pos.as_vec3());
        
        chunk_mesh.merge(&voxel_mesh).expect("invalid mesh");
    }

    // calculate collider
    let collider = Collider::from_bevy_mesh(&chunk_mesh, &ComputedColliderShape::default()).expect("invalid mesh");

    // return
    (chunk_mesh, collider)
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn startup(
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // setup noise resource
    let mut noise = FastNoiseLite::with_seed(512);
    noise.set_frequency(Some(SCALE));
    noise.set_noise_type(Some(NoiseType::Perlin));

    commands.insert_resource(Noise(Arc::new(noise)));

    // triggers the ChunkChangedEvent so that we actually spawn somewhere
    commands.trigger(ChunkChangedEvent(ChunkPosition::new(0, 0)));

    // purple test cube
    let faces_to_keep = vec![
        VoxelFace::Front,
        VoxelFace::Back,
        VoxelFace::Right,
        VoxelFace::Left,
        VoxelFace::Top,
        VoxelFace::Bottom,
    ];
    let voxel_mesh = generate_voxel_mesh(faces_to_keep);

    commands.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        Mesh3d(meshes.add(voxel_mesh)),
        MeshMaterial3d(materials.add(Color::from(PURPLE)))
    ));
}

// reacts to the SpawnChunksEvent and spawns a Task that computes all the chunks with the given chunk positions
// we allow this Task to run over several frames, when that task is complete we handle it in handle_chunks_tasks, which actually spawns the chunk
fn spawn_chunks_tasks(
    trigger: Trigger<SpawnChunksEvent>,
    mut queue: ResMut<ChunkQueue>,
    noise: Res<Noise>,
) {
    let noise = Arc::clone(&noise.0);
    let chunk_positions = Arc::new(trigger.event().0.clone());
    
    // spawn task that computes the specified chunks, returning their positions, meshes and colliders
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut data = ChunkTaskData::default();

        for chunk_pos in chunk_positions.iter() {
            let voxel_positions = compute_chunk_voxel_positions(&noise, chunk_pos);
            let (mesh, collider) = compute_chunk_mesh(&voxel_positions);

            data.voxel_positions.push(voxel_positions);
            data.transforms.push(Transform::from_xyz(
                chunk_pos.x as f32,
                0.0,
                chunk_pos.z as f32
            ));

            data.meshes.push(mesh);
            data.colliders.push(collider);
        }

        data
    });

    // push task to task queue
    queue.0.push(task);
}

// we handle SpawnChunksTasks here, checking if a given task is finished and then spawning the chunk
fn handle_chunks_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkQueue>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // debug meow
    //info!("QUEUE ITEMS: {}", queue.0.len());

    // remove tasks from the queue that have finished and have spawned successfully
    queue.0.retain_mut(|task| {
        if let Some(chunk) = block_on(future::poll_once(task)) {
            // we push Bundles into this vector because spawn_batch is faster than individually spawning
            let mut batch = Vec::with_capacity(CHUNK_SIZE_HORIZONTAL as usize);

            for (((voxel_positions, transform), mesh), collider) in chunk.voxel_positions.into_iter().zip(chunk.transforms).zip(chunk.meshes).zip(chunk.colliders) {
                batch.push(ChunkBundle(
                    Chunk {
                        voxel_positions
                    },

                    transform,
                    collider,

                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(Color::from(LAWN_GREEN))),
                ));
            }

            commands.spawn_batch(batch);

            // remove from the queue, as the task has finished 
            false
        }
        else {
            // keep it in the queue, as the task is still processing
            true
        }
    });
}

// spawns new chunks based on the player's position, runs when the ChunkChangedEvent is triggered
// triggers the SpawnChunksEvent
fn spawn_chunks_around_player(
    trigger: Trigger<ChunkChangedEvent>,
    mut commands: Commands,

    chunk_query: Query<&Transform, With<Chunk>>,
) {
    /* generate a grid of chunk positions around the player  */
    let mut new_chunks = generate_chunk_grid(&trigger.event().0);

    // get existing chunks
    let mut existing_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
    for transform in chunk_query {
        existing_chunks.insert(ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32));
    }

    // only keep the positions that dont exist, so that we dont spawn new chunks in a place where a chunk already exists
    new_chunks.retain(|pos| !existing_chunks.contains(pos));

    // spawn chunks based on chunk positions hashset
    commands.trigger(SpawnChunksEvent(new_chunks));
}

// despawns chunks that aren't in the chunk grid
// runs when the ChunkChangedEvent triggers 
fn despawn_chunks(
    trigger: Trigger<ChunkChangedEvent>,
    mut commands: Commands,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    let chunk_grid = generate_chunk_grid(&trigger.event().0);

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

        commands.trigger(ChunkChangedEvent(ChunkPosition::new(chunk_x, chunk_z)));
    }
}

fn regenerate_chunk(
    trigger: Trigger<RegenerateChunkEvent>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    chunk_query: Query<(&Chunk, &Transform)>,
) {
    let entity = trigger.event().0;
    let (chunk, chunk_transform) = match chunk_query.get(entity) {
        Ok(x) => x,
        _ => return
    };

    commands.entity(entity).despawn();

    // generate new mesh and collider based on new voxel_positions
    let (mesh, collider) = compute_chunk_mesh(&chunk.voxel_positions);

    commands.spawn(ChunkBundle(
        chunk.clone(),
        chunk_transform.clone(),
        collider,

        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::from(LAWN_GREEN))),
    ));
}