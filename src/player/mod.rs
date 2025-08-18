use bevy::prelude::*;
use crate::*;

mod movement;
mod interaction;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(movement::MovementPlugin);
        app.add_plugins(interaction::InteractionPlugin);
    }
}

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component)]
pub struct Player {
    fly: bool
}