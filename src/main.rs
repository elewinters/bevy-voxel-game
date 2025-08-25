use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod player;
mod worldgen;
mod debug;

// pub because player module reads CursorLocked so that the camera doesnt move when the mouse is unlocked
pub mod window; 

/* 
    TODO:
        - fix highlight mesh only updating position when you move the mouse (this should be easy to fix)
        - add some ambient occlusion and mess with the graphics a bit to make the game look a bit better
        - clean up a lot of the code (especially worldgen/chunk.rs and player/interaction.rs)

        MAYBE (i can always do this after 0.1.0):
            - chunk serialization
            - multiple block types

        - after all that is done, publish a 0.1.0 release on github 
*/

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

fn main() {
    let mut app = App::new();

    // bevy/ecosystem plugins
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
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