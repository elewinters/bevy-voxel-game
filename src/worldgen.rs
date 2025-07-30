use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;
use noise::*;

use crate::player;

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_atmosphere);
        app.add_systems(Startup, spawn_many_chunks.after(player::spawn_player));

        app.add_systems(Update, spawn_chunks_on_key_press);
        app.add_systems(Update, print_current_chunk);

        // spawn_single_chunk responds to SpawnChunkEvents
        app.add_observer(spawn_single_chunk);
    }
}

/* ------------------ */
/*      constants     */
/* ------------------ */
const RENDER_DISTANCE: u32 = 3;

const CHUNK_SIZE_X: u32 = 32;
const CHUNK_SIZE_Y: u32 = 1;
const CHUNK_SIZE_Z: u32 = 32;

const FLATNESS: f64 = 60.0;
const SPIKINESS: f64 = 20.0;

const CHUNKS: [[bool; 3]; 3] = [
    [true, true, true],
    [true, true, true],
    [true, true, true]
];

/* --------------- */
/*      events     */
/* --------------- */
#[derive(Event)]
struct SpawnChunkEvent {
    chunk_pos_x: f32,
    chunk_pos_z: f32
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
        Name::new(format!("CHUNK: {}, {}", event.chunk_pos_x, event.chunk_pos_z)),
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

fn spawn_many_chunks(
    mut commands: Commands,
    player_transform: Single<&Transform, With<player::Player>>
) {
    let player_x = player_transform.translation.x;
    let player_z = player_transform.translation.z;

    // this makes it so that this spawns chunks around the player (so we spawn in the center) rather than the player getting spawned on the bottom left corner of the chunk
    let start_offset = -((RENDER_DISTANCE as f32) / 2.0);

    for chunk_x in 0..RENDER_DISTANCE {
        for chunk_z in 0..RENDER_DISTANCE {
            let chunk_pos_x = player_x.round() + (CHUNK_SIZE_X as f32 * (start_offset + chunk_x as f32));
            let chunk_pos_z = player_z.round() + (CHUNK_SIZE_Z as f32 * (start_offset + chunk_z as f32));

            commands.trigger(SpawnChunkEvent {
                chunk_pos_x: chunk_pos_x,
                chunk_pos_z: chunk_pos_z
            });
        }
    }
}

fn spawn_chunks_on_key_press(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.just_pressed(KeyCode::KeyP) {
        commands.run_system_cached(spawn_many_chunks);
    }
}

fn print_current_chunk(
    chunk_query: Query<(&Name, &Transform), With<Chunk>>,
    player_transform: Single<&Transform, With<player::Player>>
) {
    let player_pos = player_transform.translation;
    
    for (chunk_name, chunk_transform) in chunk_query {
        let chunk_pos = chunk_transform.translation;
        
        // calculate chunk boundaries
        let chunk_min_x = chunk_pos.x;
        let chunk_max_x = chunk_pos.x + CHUNK_SIZE_X as f32;
        let chunk_min_z = chunk_pos.z;
        let chunk_max_z = chunk_pos.z + CHUNK_SIZE_Z as f32;
        
        // check if player is within chunk bounds
        if player_pos.x >= chunk_min_x 
            && player_pos.x < chunk_max_x
            && player_pos.z >= chunk_min_z 
            && player_pos.z < chunk_max_z {
            println!("player is in chunk: {}", chunk_name);
        }
    }
}

fn spawn_atmosphere(mut commands: Commands) {
    // blue sky
    commands.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));

    // some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 10_000.0,
        affects_lightmapped_meshes: true,
    });

    // the sun
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },

        Transform::from_xyz(0.0, 10_000.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}