use std::sync::Arc;
use std::collections::HashSet;

use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Sender, Receiver};

use bevy::prelude::*;

use fastnoise_lite::*;

use crate::player;
use super::voxel::{self, Face, VoxelData, Texture};
use super::structures;

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // add structures plugin
        app.add_plugins(structures::StructuresPlugin);
        
        // startup plugin
        app.add_systems(Startup, startup);

        // systems responsible for spawning/despawning chunks 
        app.add_systems(Update, send_chunk_messages);
        app.add_systems(Update, handle_chunk_messages);
        app.add_systems(Update, despawn_chunks);

        // resources
        app.init_resource::<ExistingChunks>();
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
const RENDER_DISTANCE: i32 = 16; // try and make sure that this number is even (cleanly divisible by 2)
const CHUNK_GRID_LEN: usize = (RENDER_DISTANCE as usize) * (RENDER_DISTANCE as usize);

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

pub struct ChunkMessage {
    transform: Transform,
    voxels: Vec<VoxelData>,
    mesh: Mesh,
}

/* ------------------ */
/*      resources     */
/* ------------------ */
// #tag resources

#[derive(Resource)]
struct GlobalMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct Noise(Arc<FastNoiseLite>);

// this is a list of all of the chunks that currently exist or will exist in the future (the thread computing them hasn't finished yet but will later)
#[derive(Resource, Default)]
struct ExistingChunks(HashSet<ChunkPosition>);

#[derive(Resource)]
struct ChunkChannel {
    sender: Sender<ChunkMessage>,
    receiver: Receiver<ChunkMessage>,
}

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct Chunk {
    pub voxels: Vec<VoxelData>
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

// round to the nearest chunk (nearest number divisible by CHUNK_SIZE_HORIZONTAL)
fn align_pos_to_chunk(x: f32) -> i32 {
    let chunk_size = CHUNK_SIZE_HORIZONTAL as f32;
    ((x / chunk_size).floor() * chunk_size) as i32
}


// this calculates a grid of chunk positions around the player based on RENDER_DISTANCE
// we spawn and despawn chunks based on this grid
fn chunk_grid(player_pos: Vec3) -> HashSet<ChunkPosition> {
    let mut new_chunks = HashSet::with_capacity(CHUNK_GRID_LEN);
    let render_half = RENDER_DISTANCE / 2;

    // get the current chunk the player is standing on
    let current_chunk = ChunkPosition::new(
        align_pos_to_chunk(player_pos.x), 
        align_pos_to_chunk(player_pos.z)
    );

    for x in -render_half..render_half {
        for z in -render_half..render_half {
            let chunk_x: i32 = current_chunk.x + (x * CHUNK_SIZE_HORIZONTAL);
            let chunk_z: i32 = current_chunk.z + (z * CHUNK_SIZE_HORIZONTAL);

            new_chunks.insert(ChunkPosition::new(chunk_x, chunk_z));
        }
    }

    new_chunks
}

fn chunk_voxels(noise: &FastNoiseLite, chunk_pos: &ChunkPosition) -> Vec<VoxelData> {
    // vector of voxels, their positions and textures and what not
    let mut voxels: Vec<VoxelData> = Vec::with_capacity(CHUNK_LEN);

    // determine position and texture of each voxel and add to voxels
    for x in 0..CHUNK_SIZE_HORIZONTAL {
        for _ in 0..CHUNK_SIZE_VERTICAL {
            for z in 0..CHUNK_SIZE_HORIZONTAL {
                let pos_x = x as f32;
                let pos_z = z as f32;

                let global_pos_x = pos_x + chunk_pos.x as f32;
                let global_pos_z = pos_z + chunk_pos.z as f32;

                // determine voxel position
                let voxel_position = Vec3::new(
                    pos_x,
                    terrain_noise(
                        noise,

                        global_pos_x,
                        global_pos_z
                    ), 
                    pos_z
                );

                // determine texture
                let dirt_patch = noise.get_noise_2d(global_pos_x, global_pos_z);
                let dirt_patch = (dirt_patch + 1.0) / 2.0; // convert to 0..1 range (get_noise_2d gives a value in the -1..1 range)

                let texture = if dirt_patch > 0.5 {
                    Texture::Dirt
                } else {
                    Texture::Grass
                };

                // add to voxels vec
                voxels.push(VoxelData::new(voxel_position.as_ivec3(), texture));
            }
        }
    }

    voxels
}

fn chunk_mesh(voxels: &Vec<VoxelData>) -> Mesh {
    // chunk mesh, initial value is essentially empty. we add individual voxels to this mesh to generate one big mesh
    let mut chunk_mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));

    // generate mesh based on voxels
    for voxel in voxels {
        let mut faces = Face::all();

        // only keep the faces that we should draw
        faces.retain(|face| voxel::should_draw_face(face, &voxel.position, voxels));

        // merge mesh
        let mut voxel_mesh = voxel::voxel_mesh(faces, &voxel.texture);
        voxel_mesh.translate_by(voxel.position.as_vec3());
        
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
    let global_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("atlas.png")),
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.insert_resource(GlobalMaterial(global_material));

    // setup chunk channel resource
    let (sender, receiver) = crossbeam_channel::unbounded();
    commands.insert_resource(ChunkChannel {sender, receiver});

    // test model
    commands.spawn((
        Name::new("bush"),
        Transform {
            translation: Vec3::new(53.0, -3.0, 28.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
            ..default()
        },

        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/bush.glb")),
        ),
    ));
}

// runs every frame and spawns new chunks around the player if they don't already exist
// we compute these chunks over multiple frames on separate threads, and once it's done computing we send a message over to handle_chunk_messages which spawns it
fn send_chunk_messages(
    channel: Res<ChunkChannel>,
    noise: Res<Noise>,
    mut existing_chunks: ResMut<ExistingChunks>,

    player_transform: Single<&Transform, With<player::Player>>,
) {
    /* generate a grid of chunk positions around the player  */
    let chunk_grid = chunk_grid(player_transform.translation);

    // spawn new chunks
    for chunk_pos in chunk_grid {
        // skip this chunk if it already exists/is being processed
        if existing_chunks.0.contains(&chunk_pos) {
            continue;
        }

        // data that'll be moved into the thread
        let noise = noise.0.clone();
        let chunk_pos_clone = chunk_pos.clone();
        let sender = channel.sender.clone();
        
        // spawns a thread that computes the specified chunk, sending it's result over the ChunkChannel once it's complete
        AsyncComputeTaskPool::get().spawn(async move {
            let transform = Transform::from_xyz(
                chunk_pos_clone.x as f32,
                0.0,
                chunk_pos_clone.z as f32
            );

            let voxels = chunk_voxels(&noise, &chunk_pos_clone);
            let mesh = chunk_mesh(&voxels);

            // we're done computing, send a message over the chunk channel
            let _ = sender.send(ChunkMessage {
                transform,

                voxels,
                mesh
            });
        })
        .detach();

        // we have started the computation thread and the chunk will spawn in the future, let's make sure this chunk doesn't accidentally get computed again in the next pass 
        existing_chunks.0.insert(chunk_pos);
    }
}

// we handle chunk messages here, spawning any chunks that we receive over the ChunkChannel
fn handle_chunk_messages(
    mut commands: Commands,
    channel: Res<ChunkChannel>,

    mut meshes: ResMut<Assets<Mesh>>,
    global_material: Res<GlobalMaterial>,
) {
    for chunk_data in channel.receiver.try_iter() {
        commands.spawn(ChunkBundle(
            Chunk {
                voxels: chunk_data.voxels
            },
            Name::new("chunk"),

            chunk_data.transform,

            Mesh3d(meshes.add(chunk_data.mesh)),
            MeshMaterial3d(global_material.0.clone()),
        ));
    }
}

// runs every frame and despawns chunks that aren't in the chunk grid
// also removes the despawned chunk from the ExistingChunks resource
fn despawn_chunks(
    mut commands: Commands,
    mut existing_chunks: ResMut<ExistingChunks>,
    
    player_transform: Single<&Transform, With<player::Player>>,
    chunk_query: Query<(Entity, &Transform), With<Chunk>>,
) {
    let chunk_grid = chunk_grid(player_transform.translation);

    // iterate over all exisiting chunks
    for (entity, transform) in chunk_query {
        let chunk_pos = ChunkPosition::new(transform.translation.x as i32, transform.translation.z as i32);

        // check chunk grid, if this chunk isn't in the grid, despawn it
        if !chunk_grid.contains(&chunk_pos) {
            commands.entity(entity).despawn();

            // can't forget to remove it from the existing chunks list!!
            existing_chunks.0.remove(&chunk_pos);
        }
    }
}