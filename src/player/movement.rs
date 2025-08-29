use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::control::KinematicCharacterController;

use crate::player::*;
use crate::window;
use crate::worldgen::chunk;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_player,
            look_player,
            fly_player
        ));
    }
}

/* ------------------ */
/*      constants     */
/* ------------------ */
// #tag constants

const MOUSE_SENSITIVITY: f32 = 0.3;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_POWER: f32 = 12.0;
const GRAVITY: f32 = -9.81;

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

// responsible for moving around with WASD, jumping and applying gravity
fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    
    player: Single<&Player>,
    player_transform: Single<&Transform, With<Player>>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
    controller_output: Option<Single<&KinematicCharacterControllerOutput>>,

    chunk_query: Query<&chunk::Chunk>,

    mut gravity: Local<f32>,
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
        input.y += JUMP_POWER;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) && player.fly {
        input.y -= JUMP_POWER;
    }

    // gravity stuff
    // do not apply gravity if we are flying
    if !player.fly {
        if let Some(x) = controller_output && x.grounded {
            *gravity = 0.0;

            // if we're jumping
            if input.y > 0.0 {
                *gravity = input.y;
            }
        }

        input.y = *gravity;
        *gravity += GRAVITY * time.delta_secs() * controller.custom_mass.expect("character controller must have a custom mass");
    }
    
    // make the player faster when flying
    if player.fly {
        controller.filter_flags = QueryFilterFlags::all();

        input.x *= 4.0;
        input.y *= 2.0;
        input.z *= 4.0;
    }
    else {
        controller.filter_flags = QueryFilterFlags::empty();
    }

    // this is where we actually move the player
    // we use the player's rotation so that we move in the direction that we're facing
    controller.translation = Some(player_transform.rotation * (input * time.delta_secs()));
}

// responsible for changing the player/camera's rotation based on mouse input, aka "looking"
fn look_player(
    mut player_transform: Single<&mut Transform, (With<Player>, Without<Camera>)>,
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    mut mouse_events: EventReader<MouseMotion>,

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

// enable flying on pressing F
fn fly_player(
    mut player: Single<&mut Player>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyF) {
        player.fly = !player.fly;
    }
}