use bevy::prelude::*;

pub mod chunk;
mod structures;
mod voxel;

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_atmosphere);

        app.add_plugins(chunk::ChunkPlugin);
        app.add_plugins(structures::StructuresPlugin);

        app.add_systems(Startup, voxel::setup_global_texture);
    }
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_atmosphere(mut commands: Commands) {
    // blue sky
    commands.insert_resource(ClearColor(Color::srgb_u8(225, 244, 244)));

    // some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 10_000.0,
        affects_lightmapped_meshes: true,
    });

    // the sun
    commands.spawn((
        Name::new("sun"),
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT * 2.0,
            shadows_enabled: true,
            ..default()
        },

        Transform::from_xyz(0.0, 10_000.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}