use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::prelude::*;
use crate::player;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_debug_menu);
        app.add_systems(Update, (
            hide_menu, 
            update_fps, 
            update_player_position
        ));

        //app.add_plugins(RapierDebugRenderPlugin::default());
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
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

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_debug_menu(mut commands: Commands) {
    commands.spawn((
        DebugMenu,
        Name::new("Debug Menu"),

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
                TextColor(Color::from(Color::BLACK)),
            ),
            (
                PlayerPositionDisplay,
                Text::new("player position: N/A"),
                TextColor(Color::from(Color::BLACK)),
            ),
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