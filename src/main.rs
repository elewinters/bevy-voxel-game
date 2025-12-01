use bevy::prelude::*;

mod player;
mod worldgen;
mod debug;

// pub because player module reads CursorLocked so that the camera doesnt move when the mouse is unlocked
pub mod window; 

/* 
    TODO:
        - structures (trees, flowers, etc. randomly spawning in the world)
        - multiple block types
        - mountains
*/

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

fn main() {
    let mut app = App::new();

    // bevy/ecosystem plugins
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        MeshPickingPlugin,
    ));

    // user plugins
    app.add_plugins((
        player::PlayerPlugin,
        window::WindowPlugin,
        worldgen::WorldGenPlugin,
        debug::DebugPlugin
    ));
    
    app.run();
}