use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::camera::Exposure;

use crate::*;

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
pub struct Player {
    fly: bool
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player { 
            fly: false 
        },

        Transform::from_xyz(0.0, 20.0, 0.0),
        Visibility::default(),
        Collider::round_cylinder(0.9, 0.3, 0.2),

        KinematicCharacterController {
            custom_mass: Some(5.0),
            
            max_slope_climb_angle: 45.0_f32.to_radians(), // don't allow climbing slopes larger than 45 degrees
            min_slope_slide_angle: 30.0_f32.to_radians(), // automatically slide down on slopes smaller than 30 degrees

            snap_to_ground: None,

            ..default()
        },

        // camera
        children![(
            Camera3d::default(), 
            Transform::from_xyz(0.0, 0.2, -0.1),
            Projection::from(PerspectiveProjection {fov: 90.0_f32.to_radians(),..default()}),

            Exposure::SUNLIGHT,
            Bloom::NATURAL,
        )]
    ));
}