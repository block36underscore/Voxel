pub mod block;
pub mod chunk;
pub mod generation;

use bevy::{math::I64Vec3, prelude::*};
use chunk::Chunk;
use generation::BlockGenerator;

#[derive(Component)]
pub struct World {
    pub generator: BlockGenerator,
}

#[derive(Component)]
pub struct ChunkOffset(I64Vec3);

pub(crate) fn propagate_chunk_offsets<const SIZE: usize>(
    mut chunks: Query<(&Chunk<SIZE>, &ChunkOffset, &mut Transform)>
) {
    chunks.par_iter_mut().for_each(|(_, offset, mut transform)| {
        *transform = Transform::from_translation((offset.0 * (SIZE as i64)).as_vec3());
    });
}

impl World {
    pub fn load<const SIZE: usize>(&self, chunk: I64Vec3) -> Chunk<SIZE> {
        Chunk::<SIZE>::generate(self.generator, chunk * SIZE as i64)
    }
}
