use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

use crate::player::*;
use crate::window;
use crate::worldgen::chunk;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_player,
            look_player,
        ));
    }
}

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

const MOUSE_SENSITIVITY: f32 = 0.3;
const MOVEMENT_SPEED: f32 = 50.0;
const ELEVATION_SPEED: f32 = 20.0;

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

// responsible for moving around with WASD, jumping and applying gravity
fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    
    mut player_transform: Single<&mut Transform, With<Player>>,
    chunk_query: Query<&chunk::Chunk>,
) {
    // do not allow player to move if the chunks have not loaded yet
    if chunk_query.is_empty() {
        return;
    }

    let mut input = Vec3::default();

    if keyboard.pressed(KeyCode::KeyW) {
        input.z -= MOVEMENT_SPEED;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input.z += MOVEMENT_SPEED;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input.x -= MOVEMENT_SPEED;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input.x += MOVEMENT_SPEED;
    }
    if keyboard.pressed(KeyCode::Space) {
        input.y += ELEVATION_SPEED;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        input.y -= ELEVATION_SPEED;
    }

    // this is where we actually move the player
    // we use the player's rotation so that we move in the direction that we're facing
    let rotation = player_transform.rotation;
    player_transform.translation += rotation * (input * time.delta_secs());
}

// responsible for changing the player/camera's rotation based on mouse input, aka "looking"
fn look_player(
    mut player_transform: Single<&mut Transform, (With<Player>, Without<Camera>)>,
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    mut mouse_events: MessageReader<MouseMotion>,

    mut look: Local<Vec2>,
    cursor_locked: Res<window::CursorLocked>
) {
    if !cursor_locked.0 {
        return;
    }

    for event in mouse_events.read() {
        look.x -= event.delta.x * MOUSE_SENSITIVITY;
        look.y -= event.delta.y * MOUSE_SENSITIVITY;
        look.y = look.y.clamp(-89.9, 89.9); // Limit pitch
    }

    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, look.x.to_radians());
    camera_transform.rotation = Quat::from_axis_angle(Vec3::X, look.y.to_radians());
}