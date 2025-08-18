use std::time::Duration;

use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};

use bevy_rapier3d::prelude::*;

mod player;
mod window;
mod worldgen;

/* 
    TODO:
        - fix highlight mesh only updating position when you move the mouse (this should be easy to fix)
        - add some ambient occlusion and mess with the graphics a bit to make the game look a bit better
        - add chunk serialization
        - clean up a lot of the code (especially worldgen/chunk.rs and player/interaction.rs)

        - after all that is done, publish a 0.1.0 release on github 
*/

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    text_color: Color::from(BLACK),
                    refresh_interval: Duration::from_millis(250),
                    enabled: true,
                },
            }
        ))
        .add_plugins((
            player::PlayerPlugin,
            window::WindowPlugin,
            worldgen::WorldGenPlugin
        ))

        .run();
}