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

/* ---------------- */
/*      systems     */
/* ---------------- */
fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let noise = Perlin::new(512);

    let size_x = 32 * 3;
    let size_y = 12;
    let size_z = 32 * 3;

    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::from(LAWN_GREEN));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_x {
                let flatness = 60.0; // the higher the number, the less hills there are
                let spikiness = 20.0; // the higher the number, the spikier hills get

                let noise_y = noise.get([x as f64 / flatness , y as f64 / flatness , z as f64 / flatness]) as f32 * spikiness;           

                if x == 0 || x == size_x - 1 ||
                   y == 0 || y == size_y - 1 ||
                   z == 0 || z == size_z - 1 {
                    commands.spawn((
                        Transform::from_xyz(x as f32, noise_y.round(), z as f32),
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