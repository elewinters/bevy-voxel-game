use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::render::camera::Exposure;

use bevy_rapier3d::control::KinematicCharacterController;

use crate::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);

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

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player { fly: false },

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
        )]
    ));
}

// responsible for moving around with WASD, jumping and applying gravity
fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    
    player: Single<&Player>,
    player_transform: Single<&Transform, With<Player>>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
    controller_output: Option<Single<&KinematicCharacterControllerOutput>>,

    mut gravity: Local<f32>,
) {
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
        input.x *= 4.0;
        input.y *= 2.0;
        input.z *= 4.0;
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

    mut look: Local<Vec2>
) {
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