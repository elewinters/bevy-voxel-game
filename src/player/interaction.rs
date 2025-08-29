use bevy::prelude::*;

use crate::worldgen::chunk;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_highlight_mesh);

        app.add_observer(highlight_voxel);
        app.add_observer(manipulate_voxels);
    }
}

/* ------------------- */
/*      components     */
/* ------------------- */
// #tag components

#[derive(Component)]
struct HighlightMesh;

/* ------------------ */
/*      functions     */
/* ------------------ */
// #tag functions

// align the hit position from the Pointer<Click> event to the voxel_positions vector
fn align_hit_pos_inward(local_pos: Vec3, normal: Vec3) -> IVec3 {
    // move slightly inward from the surface to ensure we're inside the voxel
    (local_pos - normal * 0.1).round().as_ivec3()
}

fn align_hit_pos_outward(local_pos: Vec3, normal: Vec3) -> IVec3 {
    // move slightly inward from the surface to ensure we're inside the voxel
    (local_pos + normal * 0.1).round().as_ivec3()
}

/* ---------------- */
/*      systems     */
/* ---------------- */
// #tag systems

fn spawn_highlight_mesh(
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        HighlightMesh,
        Name::new("highlight mesh"),
        Pickable::IGNORE,

        Transform::from_xyz(0.0, 10.0, 0.0),
        Mesh3d(meshes.add(Cuboid::from_length(1.01))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.1, 0.1, 0.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }))
    ));
}

fn highlight_voxel(
    trigger: Trigger<Pointer<Move>>,

    chunk_query: Query<Entity, (With<chunk::Chunk>, Without<HighlightMesh>)>,
    mut highlight_transform: Single<&mut Transform, (With<HighlightMesh>, Without<chunk::Chunk>)>,
) {
    // get event
    let event = trigger.event();
    let entity = event.target;

    // check if hit is a chunk
    if chunk_query.get(entity).is_err() {
        return;
    }

    // event data hit position
    let (pos, normal) = match (event.hit.position, event.hit.normal) {
        (Some(pos), Some(normal)) => (pos, normal),
        _ => return
    };

    let pos = align_hit_pos_inward(pos, normal);
    highlight_transform.translation = pos.as_vec3();
}

// voxel breaking/placing
fn manipulate_voxels(
    trigger: Trigger<Pointer<Click>>,

    mut commands: Commands,
    mut chunk_query: Query<(&mut chunk::Chunk, &Transform)>,
) {
    // get event
    let event = trigger.event();
    let entity = event.target;
    
    // the chunk that we hit and its transform
    let (mut chunk, chunk_transform) = match chunk_query.get_mut(entity){
        Ok(x) => x,
        Err(_) => return // entity not in chunk_query, we return as that means that whatever we clicked on isnt a chunk
    };

    // event data hit position
    let (pos, normal) = match (event.hit.position, event.hit.normal) {
        (Some(pos), Some(normal)) => (pos, normal),
        _ => return
    };

    // convert hit position to local chunk space
    let local_pos = pos - chunk_transform.translation;

    match trigger.button {
        // voxel breaking
        PointerButton::Primary => {
            let pos = align_hit_pos_inward(local_pos, normal);

            // remove hit voxel from voxel_positions
            // dont regenerate chunk if block doesn't exist
            if !chunk.voxel_positions.remove(&pos) {
                return;
            }
        },
        // voxel placing
        PointerButton::Secondary => {
            let pos = align_hit_pos_outward(local_pos, normal);

            // add new voxel position to voxel_positions
            // dont regenerate chunk if it already exists
            if !chunk.voxel_positions.insert(pos) {
                return;
            }
        },
        _ => return
    }

    // regenerate chunk
    commands.trigger(chunk::RegenerateChunk(entity))
}