use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

use crate::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);

        app.add_systems(Update, (
            player_look,
            player_movement
        ));
    }
}

const MOUSE_SENSITIVITY: f32 = 0.3;
const GROUND_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_POWER: f32 = 20.0;
const GRAVITY: f32 = -9.81;

#[derive(Component)]
struct Player;

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,

        Transform::from_xyz(0.0, 5.0, 0.0),
        Visibility::default(),
        Collider::round_cylinder(0.9, 0.3, 0.2),

        KinematicCharacterController {
            custom_mass: Some(5.0),
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Relative(0.3),
                min_width: CharacterLength::Relative(0.5),
                include_dynamic_bodies: false,
            }),
            
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
        )]
    ));
}

fn player_movement(
    time: Res<Time>,

    keyboard: Res<ButtonInput<KeyCode>>,
    
    player_transform: Single<&Transform, With<Player>>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
    controller_output: Option<Single<&KinematicCharacterControllerOutput>>,

    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
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
        input.y = JUMP_POWER;
    }

    // Check physics ground check
    if let Some(x) = controller_output && x.grounded {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
        
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= time.delta_secs();
        // If we jump we clear the grounded tolerance
        if input.y > 0.0 {
            *vertical_movement = input.y;
            *grounded_timer = 0.0;
        }
    }
    input.y = *vertical_movement;
    *vertical_movement += GRAVITY * time.delta_secs() * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(player_transform.rotation * (input * time.delta_secs()));
}

fn player_look(
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