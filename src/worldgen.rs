use bevy::prelude::*;
use bevy::color::palettes::css::*;

use bevy_rapier3d::prelude::*;

use bevy::render::{
    mesh::Indices,
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

use noise::*;

pub struct WorldGenPlugin;
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_terrain, spawn_atmosphere));
    }
}

#[derive(Debug, Clone, Copy)]
enum BlockFace {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Back = 4,
    Forward = 5
}

/* ------------------ */
/*      constants     */
/* ------------------ */
const FLATNESS: f64 = 60.0;
const SPIKINESS: f64 = 20.0;

/* ------------------ */
/*      functions     */
/* ------------------ */
fn generate_noise(x: f64, y: f64, z: f64) -> f32 {
    let noise = Perlin::new(512);
    let noise_y = noise.get([x / FLATNESS , y / FLATNESS , z / FLATNESS]) * SPIKINESS;

    noise_y.round() as f32
}

fn create_cube_mesh<const N: usize>(without_faces: [BlockFace; N]) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);

    let mut positions = vec![
        // top (facing towards +y)
        [-0.5, 0.5, -0.5], // vertex with index 0
        [0.5, 0.5, -0.5], // vertex with index 1
        [0.5, 0.5, 0.5], // etc. until 23
        [-0.5, 0.5, 0.5],
        // bottom   (-y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // right    (+x)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
        [0.5, 0.5, -0.5],
        // left     (-x)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // back     (+z)
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        // forward  (-z)
        [-0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
    ];

    let mut uvs = vec![
        // Assigning the UV coords for the top side.
        [0.0, 0.2], [0.0, 0.0], [1.0, 0.0], [1.0, 0.2],
        // Assigning the UV coords for the bottom side.
        [0.0, 0.45], [0.0, 0.25], [1.0, 0.25], [1.0, 0.45],
        // Assigning the UV coords for the right side.
        [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
        // Assigning the UV coords for the left side.
        [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
        // Assigning the UV coords for the back side.
        [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
        // Assigning the UV coords for the forward side.
        [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
    ];

    let mut normals = vec![
        // Normals for the top side (towards +y)
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Normals for the bottom side (towards -y)
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Normals for the right side (towards +x)
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Normals for the left side (towards -x)
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Normals for the back side (towards +z)
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Normals for the forward side (towards -z)
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ];

    let mut indices = vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y)
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ];

    // remove all the faces that we don't want
    for face in without_faces {
        for i in 0..=5 {
            if face as usize == 0 {
                indices.remove(0);
            }
            else {
                println!("INDIC{}", face as usize);
                println!("{}", indices.remove(face as usize * 6));
            }
        }
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        positions
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        uvs
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
    );

    mesh.insert_indices(Indices::U32(indices));

    mesh
}


/* ---------------- */
/*      systems     */
/* ---------------- */
fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let size_x = 32 * 3;
    let size_y = 12;
    let size_z = 32 * 3;

    let mesh = meshes.add(create_cube_mesh([]));
    let material = materials.add(Color::from(LAWN_GREEN));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_x {
                if x == 0 || x == size_x - 1 ||
                   y == 0 || y == size_y - 1 ||
                   z == 0 || z == size_z - 1 {
                    commands.spawn((
                        Transform::from_xyz(x as f32, generate_noise(x as f64, y as f64, z as f64), z as f32),
                        Collider::cuboid(0.5, 0.5, 0.5),

                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                    ));
                }
            }
        }
    }

    commands.spawn((
        Transform::from_xyz(50.0, 8.0, 50.0),
        Collider::cuboid(0.5, 0.5, 0.5),

        Mesh3d(meshes.add(create_cube_mesh([BlockFace::Top, BlockFace::Bottom]))),
        MeshMaterial3d(materials.add(Color::from(HOT_PINK))),
    ));
}

fn spawn_atmosphere(mut commands: Commands) {
    // blue sky
    commands.insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)));

    // some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 10_000.0,
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