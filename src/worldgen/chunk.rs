use std::sync::{Arc, Mutex};
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

        // spawn_chunk responds to SpawnChunkEvents
        app.add_event::<SpawnChunksEvent>();
        app.add_systems(Update, (spawn_chunk_tasks, handle_chunk_tasks));

        // and all of these respond to ChunkChangedEvents
        app.add_event::<ChunkChangedEvent>();
        app.add_systems(Update, (spawn_chunks, despawn_chunks));
    }
}
/* 
    TODO:
        - fix face culling (later though)
        - add multithreading to chunk generation

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

/* --------------- */
/*      events     */
/* --------------- */
// #tag events

#[derive(Event)]
struct SpawnChunksEvent(Vec<ChunkPosition>);

// the chunk that the player is currently standing on has changed
// ChunkPosition represents the coordinates of the new chunk that we've stepped on
#[derive(Event)]
struct ChunkChangedEvent(ChunkPosition);


/* ------------------ */
/*      resources     */
/* ------------------ */
// #tag resources

#[derive(Resource)]
struct Noise(Arc<FastNoiseLite>);

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

// the hashset represents voxel position data
// the player can destroy a block by aligning their mouse position to the voxel position and then removing the position from the voxels field
// after that the mesh gets reconstructed based on the voxels field, meaning that the destroyed block wont be there anymore
#[derive(Component)]
#[require(Transform, Visibility)]
struct Chunk;

#[derive(Component)]
struct SpawnChunksTask(Task<(Vec<ChunkPosition>, Vec<Mesh>, Vec<Collider>)>);

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

    noise_y.round()
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
        
        // the bottom of all terrain is never seen so we just always return false here
        VoxelFace::Bottom => return false,
    };
    
    !voxel_positions.contains(&neighbor_pos)
}

fn compute_chunk(noise: &FastNoiseLite, chunk_pos: &ChunkPosition) -> (ChunkPosition, Mesh) {
    // hashset of voxel positions
    let mut voxel_positions = HashSet::new();

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

    // chunk mesh, initial value is essentially empty. we add individual voxels to this mesh to generate one big mesh
    let mut chunk_mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));

    // generate mesh based on voxels
    for voxel_pos in &voxel_positions {
        // determine which faces to keep for this mesh
        let mut faces_to_keep = Vec::new();
        
        // check each face
        if should_draw_face(VoxelFace::Front, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Front);
        }
        if should_draw_face(VoxelFace::Back, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Back);
        }
        if should_draw_face(VoxelFace::Right, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Right);
        }
        if should_draw_face(VoxelFace::Left, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Left);
        }
        if should_draw_face(VoxelFace::Top, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Top);
        }
        if should_draw_face(VoxelFace::Bottom, voxel_pos, &voxel_positions) {
            faces_to_keep.push(VoxelFace::Bottom);
        }

        // merge mesh
        let mut voxel_mesh = generate_voxel_mesh(faces_to_keep);
        voxel_mesh.translate_by(voxel_pos.as_vec3());
        
        chunk_mesh.merge(&voxel_mesh).expect("invalid mesh");
    }

    (chunk_pos.clone(), chunk_mesh)
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn startup(
    mut commands: Commands,
    mut chunk_changed: EventWriter<ChunkChangedEvent>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // setup noise resource
    let mut noise = FastNoiseLite::with_seed(512);
    noise.set_frequency(Some(SCALE));
    noise.set_noise_type(Some(NoiseType::Perlin));

    commands.insert_resource(Noise(Arc::new(noise)));

    // triggers the ChunkChangedEvent so that we actually spawn somewhere
    chunk_changed.write(ChunkChangedEvent(ChunkPosition::new(0, 0)));

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

// spawn a chunk at the specified position when a SpawnChunksEvent is fired
fn spawn_chunk_tasks(
    mut spawn_chunks: EventReader<SpawnChunksEvent>,
    noise: Res<Noise>,

    par_commands: ParallelCommands,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    spawn_chunks.par_read().for_each(|event| {
        let noise = Arc::clone(&noise.0);
        let chunk_positions = Arc::new(event.0.clone());
        
        let task = thread_pool.spawn(async move {
            println!("computing...");
            let mut positions = Vec::new();
            let mut meshes = Vec::new();
            let mut colliders = Vec::new();

            for chunk_pos in chunk_positions.iter() {
                let (position, mesh) = compute_chunk(&noise, &chunk_pos);

                positions.push(position);
                colliders.push(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::default()).expect("invalid mesh"));
                meshes.push(mesh);
            }

            (positions, meshes, colliders)
        });

        par_commands.command_scope(|mut commands| {
            commands.spawn(SpawnChunksTask(task));
        });
    });
}

fn handle_chunk_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SpawnChunksTask)>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some((chunk_positions, chunk_meshes, chunk_colliders)) = block_on(future::poll_once(&mut task.0)) {
            println!("computed!");
            let mut batch = Vec::new();
            for ((position, mesh), collider) in chunk_positions.iter().zip(chunk_meshes).zip(chunk_colliders) {
                batch.push((
                    Chunk,

                    Transform::from_xyz(
                        position.x as f32,
                        0.0,
                        position.z as f32
                    ),

                    collider,

                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(Color::from(LAWN_GREEN))),
                ));
            }

            commands.spawn_batch(batch);

            // task is complete, so remove task component from entity
            commands.entity(entity).remove::<SpawnChunksTask>();
        }
    }
}

// spawns new chunks based on the player's position
fn spawn_chunks(
    mut chunk_changed: EventReader<ChunkChangedEvent>,
    spawn_chunk: EventWriter<SpawnChunksEvent>,

    chunk_query: Query<&Transform, With<Chunk>>,
) {
    let spawn_chunks = Mutex::new(spawn_chunk);

    chunk_changed.par_read().for_each(|current_chunk| {
        /* generate a grid of chunk positions around the player  */
        let new_chunks = generate_chunk_grid(&current_chunk.0);

        // get existing chunks
        let mut existing_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
        for transform in chunk_query {
            existing_chunks.insert(ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32));
        }


        let mut positions = Vec::new();

        // spawn new chunks based on the grid in new_chunks
        for pos in new_chunks {
            // as long as it doesn't exist already
            if existing_chunks.contains(&pos) {
                continue;
            }

            positions.push(pos);
        }

        let mut spawn_chunks_mtx = spawn_chunks.lock().unwrap();
        spawn_chunks_mtx.write(SpawnChunksEvent(positions));
    });
}

fn despawn_chunks(
    mut chunk_changed: EventReader<ChunkChangedEvent>,
    par_commands: ParallelCommands,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    chunk_changed.par_read().for_each(|current_chunk| {
        let chunks = generate_chunk_grid(&current_chunk.0);

        // check all existing chunks
        for (entity, transform) in chunk_query {
            // if this chunk isn't in the grid, despawn it
            if !chunks.contains(&ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32)) {
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                });
            }
        }
    });
}

fn update_current_chunk(
    mut chunk_changed: EventWriter<ChunkChangedEvent>,
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

        chunk_changed.write(ChunkChangedEvent(ChunkPosition::new(chunk_x, chunk_z)));
    }
}