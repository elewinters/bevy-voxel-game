use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;

use noise::*;

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_terrain, spawn_atmosphere));
    }
}

const FLATNESS: f64 = 60.0;
const SPIKINESS: f64 = 20.0;

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
fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let size_x = 32 * 3;
    let size_y = 12;
    let size_z = 32 * 3;

    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::from(LAWN_GREEN));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_x {
                if x == 0 || x == size_x - 1 ||
                   y == 0 || y == size_y - 1 ||
                   z == 0 || z == size_z - 1 {
                    commands.spawn((
                        Transform::from_xyz(x as f32, generate_noise(x as f64, y as f64, z as f64), z as f32),
                        Collider::cuboid(0.5, 0.5, 0.5),

                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                    ));
                }
            }
        }
    }
}

fn spawn_atmosphere(mut commands: Commands) {
    // blue sky
    commands.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));

    // some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 10000.0,
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