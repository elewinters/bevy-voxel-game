use bevy::prelude::*;
use crate::*;

mod movement;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(movement::MovementPlugin);
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