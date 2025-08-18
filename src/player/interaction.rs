use bevy::prelude::*;

use crate::player::*;
use crate::worldgen::chunk;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(break_voxel);
    }
}

fn break_voxel(
    trigger: Trigger<Pointer<Click>>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut chunk_query: Query<(&mut chunk::Chunk, &Transform)>,
) {
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
    let voxel_pos = (local_pos - normal * 0.1).round().as_ivec3();  // move slightly inward from the surface to ensure we're inside the voxel

    // remove hit voxel from voxel_positions, dont regenerate mesh if block doesn't exist
    if !chunk.voxel_positions.remove(&voxel_pos) {
        return;
    }

    // despawn mesh and collider
    commands.entity(entity).remove::<Mesh3d>();
    commands.entity(entity).remove::<Collider>();

    // generate new mesh and collider based on new voxel_positions
    let (mesh, collider) = chunk::compute_chunk_mesh(&chunk.voxel_positions);
    commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
    commands.entity(entity).insert(collider);
}