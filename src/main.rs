use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};

mod player;
mod window;

/* ------------------ */
/*      functions     */
/* ------------------ */
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            player::PlayerPlugin,
            window::WindowPlugin
        ))

        .add_systems(Startup, spawn_map)
        .run();
}

/* ---------------- */
/*      systems     */
/* ---------------- */
fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground
    commands.spawn((
        Transform::from_xyz(0.0, -0.1, 0.0),
        Collider::cuboid(50.0, 0.1, 50.0),
    ));

    // spawn a plane of cubes in 3 dimensions

    for x in 0..50 {
        for y in 0..5 {
            for z in 0..50 {
                commands.spawn((
                    Transform::from_xyz(x as f32, y as f32, z as f32),

                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    Collider::cuboid(0.5, 0.5, 0.5),

                    MeshMaterial3d(materials.add(Color::from(LAWN_GREEN))),
                ));
            }
        }
    }
}