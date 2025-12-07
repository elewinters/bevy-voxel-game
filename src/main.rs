use bevy::prelude::*;

mod player;
mod worldgen;
mod debug;

// pub because player module reads CursorLocked so that the camera doesnt move when the mouse is unlocked
pub mod window; 

/* 
    TODO:
        optimize structures by using instancing
        randomly rotate structures like rocks and sticks to make them look Better 
        add fog shader
        add rocks and snow to mountains instead of grass and dirt

        fix holes in steep mountains

        MAYBE:
            add audio (birds chirping, footsteps, wind, etc..)
            physics
            main menu (with graphics options)

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