use bevy::prelude::*;
use bevy::window::*;

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (window_setup, cursor_lock));
        app.add_systems(Update, cursor_locking_on_esc);
    }
}

// center the window
fn window_setup(mut window: Single<&mut Window>) {
    window.position = WindowPosition::Centered(MonitorSelection::Current);
    window.present_mode = PresentMode::AutoNoVsync;
}

// enables us to unlock the cursor when we press esc
fn cursor_locking_on_esc(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,

    mut switch: Local<bool>
) {
    if keys.just_pressed(KeyCode::Escape) {
        *switch = !(*switch);

        if *switch {
            commands.run_system_cached(cursor_unlock);
        }
        else {
            commands.run_system_cached(cursor_lock);
        }
    }
}

// locks the cursor (runs on startup)
fn cursor_lock(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

// unlocks the cursor (ran by unlock_cursor_on_esc)
fn cursor_unlock(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}