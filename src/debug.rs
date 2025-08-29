use bevy::input::common_conditions::input_toggle_active;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::prelude::*;
use crate::player;
use crate::worldgen::chunk;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_menu);
        app.add_systems(Update, (
            hide_menu,

            update_fps,
            update_player_position,
            update_chunk_queue
        ));

        app.add_observer(update_current_chunk);

        //app.add_plugins(RapierDebugRenderPlugin::default());
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Tab)),
        ));
    }
}

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component)]
pub struct DebugMenu;

#[derive(Component)]
pub struct FpsDisplay;

#[derive(Component)]
pub struct PlayerPositionDisplay;

#[derive(Component)]
pub struct CurrentChunkDisplay;

#[derive(Component)]
pub struct ChunkQueueDisplay;

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_debug_menu(mut commands: Commands) {
    commands.spawn((
        DebugMenu,
        Name::new("debug menu"),

        Visibility::Hidden,

        Node {
            display: Display::Grid,
            position_type: PositionType::Absolute,

            align_items: AlignItems::Center,

            top: Val::Px(25.0),
            right: Val::Px(50.0),
            ..default()
        },

        children![
            (
                FpsDisplay,
                Text::new("FPS: N/A"),
                TextColor(Color::BLACK),
            ),
            (
                PlayerPositionDisplay,
                Text::new("player position: N/A"),
                TextColor(Color::BLACK),
            ),
            (
                CurrentChunkDisplay,
                Text::new("current chunk: N/A"),
                TextColor(Color::BLACK)
            ),
            (
                ChunkQueueDisplay,
                Text::new("chunk queue: N/A"),
                TextColor(Color::BLACK)
            )
        ],
    ));
}

// hides/shows the debug menu when TAB is pressed
fn hide_menu(
    mut menu: Single<&mut Visibility, With<DebugMenu>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        menu.toggle_inherited_hidden();
    }
}

// update fps display
fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut display: Single<&mut Text, With<FpsDisplay>>
) {
    let fps = match diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        Some(fps) => fps.smoothed().unwrap_or(0.0),
        None => return
    };

    **display = Text::new(format!("FPS: {}", fps.trunc()));
}


// update player position display
fn update_player_position(
    player_position: Single<&Transform, With<player::Player>>,
    mut display: Single<&mut Text, With<PlayerPositionDisplay>>
) {
    **display = Text::new(format!("player position: {}", player_position.translation.trunc()))
}

fn update_current_chunk(
    event: Trigger<chunk::ChunkChanged>,
    mut display: Single<&mut Text, With<CurrentChunkDisplay>>
) {
    let pos = event.event().0.clone();
    **display = Text::new(format!("current chunk: [{}, {}]", pos.x, pos.z))
}

fn update_chunk_queue(
    chunk_queue: Res<chunk::ChunkQueue>,
    mut display: Single<&mut Text, With<ChunkQueueDisplay>>
) {
    **display = Text::new(format!("chunk queue: {}", chunk_queue.0.len()))
}