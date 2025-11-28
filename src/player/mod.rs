use bevy::prelude::*;
use bevy::camera::Exposure;

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::pbr::{ScreenSpaceAmbientOcclusion};

mod movement;
mod interaction;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);

        app.add_plugins(movement::MovementPlugin);
        app.add_plugins(interaction::InteractionPlugin);
    }
}

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component)]
pub struct Player;
/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Name::new("player"),

        Transform::from_xyz(0.0, 20.0, 0.0),
        Visibility::default(),

        // camera
        children![(
            Camera3d::default(),

            Msaa::Off,
            ScreenSpaceAmbientOcclusion {
                constant_object_thickness: 10.0,
                ..default()
            },
            TemporalAntiAliasing::default(),

            Transform::from_xyz(0.0, 0.2, -0.1),
            Projection::from(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }),

            Exposure::SUNLIGHT,
        )]
    ));
}