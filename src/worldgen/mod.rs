use bevy::prelude::*;

pub mod chunk;
mod voxel;

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_atmosphere);
        app.add_plugins(chunk::ChunkPlugin);
    }
}

fn spawn_atmosphere(mut commands: Commands) {
    // blue sky
    commands.insert_resource(ClearColor(Color::srgb_u8(225, 244, 244)));

    // some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 5_000.0,
        affects_lightmapped_meshes: true,
    });

    // the sun
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },

        Transform::from_xyz(0.0, 10_000.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}