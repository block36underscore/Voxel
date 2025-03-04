pub mod block;
pub mod chunk;
pub mod generation;

use bevy::{math::I64Vec3, prelude::*};
use chunk::Chunk;
use generation::BlockGenerator;

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Level {
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

impl Level {
    pub fn load<const SIZE: usize>(&self, chunk: I64Vec3) -> (Chunk<SIZE>, ChunkOffset) {
        (
            Chunk::<SIZE>::generate(self.generator, chunk * SIZE as i64),
            ChunkOffset(chunk),
        )
    }
}

pub trait Load {
    fn load<const SIZE: usize>(&mut self, chunk: I64Vec3, commands: &mut Commands);
}

impl Load for (Entity, &Level) {
    fn load<const SIZE: usize>(&mut self, chunk: I64Vec3, commands: &mut Commands) {
        let chunk = commands.spawn((self.1.load::<SIZE>(chunk), Visibility::Visible)).id();
        // commands.entity(self.0).add_child(chunk);
    }
}
