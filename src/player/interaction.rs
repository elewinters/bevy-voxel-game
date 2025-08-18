use bevy::prelude::*;

use crate::worldgen::chunk;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_highlight_mesh);

        app.add_observer(highlight_voxel);
        app.add_observer(break_voxel);
        app.add_observer(place_voxel);
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
    let mesh = Cuboid::from_length(1.01);

    commands.spawn((
        HighlightMesh,
        Pickable::IGNORE,

        Transform::from_xyz(0.0, 10.0, 0.0),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.1, 0.1, 0.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }))
    ));
}

fn highlight_voxel(
    trigger: Trigger<Pointer<Move>>,

    chunk_query: Query<&Transform, (With<chunk::Chunk>, Without<HighlightMesh>)>,
    mut highlight_transform: Single<&mut Transform, (With<HighlightMesh>, Without<chunk::Chunk>)>,
) {
    // get event
    let event = trigger.event();

    // the chunk that we hit and its entity and transform
    let entity = event.target;
    let chunk_transform = match chunk_query.get(entity){
        Ok(transform) => transform,
        Err(_) => return // entity not in chunk_query, we return as that means that whatever we clicked on isnt a chunk
    };

    // event data hit position
    let (pos, normal) = match (event.hit.position, event.hit.normal) {
        (Some(pos), Some(normal)) => (pos, normal),
        _ => return
    };

    // convert hit position to local chunk space
    let local_pos = pos - chunk_transform.translation;
    let pos = align_hit_pos_inward(local_pos, normal);

    highlight_transform.translation = pos.as_vec3() + chunk_transform.translation;
}

fn break_voxel(
    trigger: Trigger<Pointer<Click>>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut chunk_query: Query<(&mut chunk::Chunk, &Transform)>,
) {
    // only respond to left clicks
    match trigger.button {
        PointerButton::Primary => (),
        _ => return
    }

    // get event
    let event = trigger.event();

    // the chunk that we hit and its entity and transform
    let entity = event.target;
    let (mut chunk, chunk_transform) = match chunk_query.get_mut(entity){
        Ok((chunk, transform)) => (chunk, transform),
        Err(_) => return // entity not in chunk_query, we return as that means that whatever we clicked on isnt a chunk
    };

    // event data hit position
    let (pos, normal) = match (event.hit.position, event.hit.normal) {
        (Some(pos), Some(normal)) => (pos, normal),
        _ => return
    };

    // convert hit position to local chunk space
    let local_pos = pos - chunk_transform.translation;
    let pos = align_hit_pos_inward(local_pos, normal);

    // remove hit voxel from voxel_positions, dont regenerate mesh if block doesn't exist
    if !chunk.voxel_positions.remove(&pos) {
        return;
    }

    // regenerate chunk
    chunk::regenerate_chunk(&mut commands.entity(entity), &mut meshes, &chunk.voxel_positions);
}

fn place_voxel(
    trigger: Trigger<Pointer<Click>>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut chunk_query: Query<(&mut chunk::Chunk, &Transform)>,
) {
    // only respond to right clicks
    match trigger.button {
        PointerButton::Secondary => (),
        _ => return
    }

    // get event
    let event = trigger.event();

    // the chunk that we hit and its entity and transform
    let entity = event.target;
    let (mut chunk, chunk_transform) = match chunk_query.get_mut(entity){
        Ok((chunk, transform)) => (chunk, transform),
        Err(_) => return // entity not in chunk_query, we return as that means that whatever we clicked on isnt a chunk
    };

    // event data hit position
    let (pos, normal) = match (event.hit.position, event.hit.normal) {
        (Some(pos), Some(normal)) => (pos, normal),
        _ => return
    };

    // convert hit position to local chunk space
    let local_pos = pos - chunk_transform.translation;
    let pos = align_hit_pos_outward(local_pos, normal);

    // add new voxel position to voxel_positions
    chunk.voxel_positions.insert(pos);

    // regenerate chunk
    chunk::regenerate_chunk(&mut commands.entity(entity), &mut meshes, &chunk.voxel_positions);
}