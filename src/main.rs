use bevy::prelude::*;
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};

mod player;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0))) /* white skybox */
        
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(player::PlayerPlugin)
        .add_systems(Startup, setup_map)
        .run();
}

fn setup_map(mut commands: Commands) {
    // ground
    let ground_size = 50.0;
    let ground_height = 0.1;

    commands.spawn((
        Transform::from_xyz(0.0, -ground_height, 0.0),
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    // stairs
    let stair_len = 30;
    let stair_step = 1.0;

    for i in 1..=stair_len {
        let step = i as f32;
        let collider = Collider::cuboid(1.0, step * stair_step, 1.0);
        commands.spawn((
            Transform::from_xyz(40.0, step * stair_step, step * 2.0 - 20.0),
            collider.clone(),
        ));
        commands.spawn((
            Transform::from_xyz(-40.0, step * stair_step, step * -2.0 + 20.0),
            collider.clone(),
        ));
        commands.spawn((
            Transform::from_xyz(step * 2.0 - 20.0, step * stair_step, 40.0),
            collider.clone(),
        ));
        commands.spawn((
            Transform::from_xyz(step * -2.0 + 20.0, step * stair_step, -40.0),
            collider.clone(),
        ));
    }
}