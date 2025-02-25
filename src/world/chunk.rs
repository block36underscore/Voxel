use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use bevy::{math::I64Vec3, prelude::*, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, Render, RenderApp}};

use crate::render::buffers::{self, update_buffers, Cube, ToCubes};

use super::{block::BlockState, generation::BlockGenerator, propagate_chunk_offsets};

#[derive(Default)]
pub struct ChunkPlugin<const SIZE: usize>;

impl <const SIZE: usize> Plugin for ChunkPlugin<SIZE> {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<Chunk<SIZE>>::default());
        app.add_systems(PostUpdate, propagate_chunk_offsets::<SIZE>);
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_systems(Render, buffers::update_chunk_buffers::<SIZE>.after(update_buffers));
    }
}

const fn index_to_pos(index: usize, size: usize) -> IVec3 {
    IVec3::new(
        (index / (size * size)) as i32,
        ((index / size) % size) as i32,
        (index % size) as i32,
    )
}

#[derive(Debug, Component)]
pub struct Chunk<const SIZE: usize> {
    pub blocks: [[[BlockState; SIZE]; SIZE]; SIZE],
}

impl<const SIZE: usize> Chunk<SIZE> {
    fn to_extracted(&self) -> ExtractedChunk<SIZE> {
        ExtractedChunk {
            blocks: self.blocks.clone(),
        }
    }

    pub fn generate(generator: BlockGenerator, offset: I64Vec3) -> Self {
        let mut output = Self::default();

        for i in 0..Self::volume() {
            let pos = offset + Self::index_to_pos(i).as_i64vec3();
            output[i] = generator(pos);
        }

        output
    }

    pub fn volume() -> usize {
        SIZE*SIZE*SIZE
    }

    pub fn index_to_pos(index: usize) -> IVec3 {
        index_to_pos(index, SIZE)
    }
}

impl<const SIZE: usize> Default for Chunk<SIZE> {
    fn default() -> Self {
        Self {
            blocks: [[[false; SIZE]; SIZE]; SIZE],
        }
    }
}

impl<'a, const SIZE: usize> IntoIterator for &'a Chunk<SIZE> {
    type Item = &'a BlockState;
    type IntoIter = Iter<'a, BlockState>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.as_flattened().as_flattened().iter()
    }
}

impl<'a, const SIZE: usize> IntoIterator for &'a mut Chunk<SIZE> {
    type Item = &'a mut BlockState;
    type IntoIter = IterMut<'a, BlockState>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.as_flattened_mut().as_flattened_mut().iter_mut()
    }
}

impl<'a, const SIZE: usize> Index<usize> for Chunk<SIZE> {
    type Output = BlockState;

    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks.as_flattened().as_flattened()[index]
    }
}

impl<'a, const SIZE: usize> IndexMut<usize> for Chunk<SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks.as_flattened_mut().as_flattened_mut()[index]
    }
}

impl<'a, const SIZE: usize> Index<IVec3> for Chunk<SIZE> {
    type Output = BlockState;

    fn index(&self, index: IVec3) -> &Self::Output {
        let index = index.x as usize * SIZE * SIZE + index.y as usize * SIZE + index.z as usize;
        &self.blocks.as_flattened().as_flattened()[index]
    }
}

impl<'a, const SIZE: usize> IndexMut<IVec3> for Chunk<SIZE> {
    fn index_mut(&mut self, index: IVec3) -> &mut Self::Output {
        let index = index.x as usize * SIZE * SIZE + index.y as usize * SIZE + index.z as usize;
        &mut self.blocks.as_flattened_mut().as_flattened_mut()[index]
    }
}

pub type Chunk16 = Chunk<16>;
pub type Chunk32 = Chunk<32>;

#[derive(Debug, Component)]
pub struct ExtractedChunk<const SIZE: usize> {
    pub blocks: [[[BlockState; SIZE]; SIZE]; SIZE],
}

impl<const SIZE: usize> ExtractComponent for Chunk<SIZE> {
    type QueryData = (&'static Self, &'static Transform, &'static ViewVisibility);
    type QueryFilter = ();
    type Out = (ExtractedChunk<SIZE>, Transform, ViewVisibility);

    fn extract_component(
        item: bevy::ecs::query::QueryItem<'_, Self::QueryData>,
    ) -> Option<Self::Out> {
        let (chunk, transform, visibility) = item;
        Some((chunk.to_extracted(), *transform, *visibility))
    }
}

impl<'a, const SIZE: usize> IntoIterator for &'a ExtractedChunk<SIZE> {
    type Item = &'a BlockState;
    type IntoIter = Iter<'a, BlockState>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.as_flattened().as_flattened().iter()
    }
}

impl<'a, const SIZE: usize> IntoIterator for &'a mut ExtractedChunk<SIZE> {
    type Item = &'a mut BlockState;
    type IntoIter = IterMut<'a, BlockState>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.as_flattened_mut().as_flattened_mut().iter_mut()
    }
}

impl<'a, const SIZE: usize> Index<usize> for ExtractedChunk<SIZE> {
    type Output = BlockState;

    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks.as_flattened().as_flattened()[index]
    }
}

impl<'a, const SIZE: usize> IndexMut<usize> for ExtractedChunk<SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks.as_flattened_mut().as_flattened_mut()[index]
    }
}

impl<'a, const SIZE: usize> Index<IVec3> for ExtractedChunk<SIZE> {
    type Output = BlockState;

    fn index(&self, index: IVec3) -> &Self::Output {
        let index = index.x as usize * SIZE * SIZE + index.y as usize * SIZE + index.z as usize;
        &self.blocks.as_flattened().as_flattened()[index]
    }
}

impl<'a, const SIZE: usize> IndexMut<IVec3> for ExtractedChunk<SIZE> {
    fn index_mut(&mut self, index: IVec3) -> &mut Self::Output {
        let index = index.x as usize * SIZE * SIZE + index.y as usize * SIZE + index.z as usize;
        &mut self.blocks.as_flattened_mut().as_flattened_mut()[index]
    }
}

impl<const SIZE: usize> ToCubes for ExtractedChunk<SIZE> {
    fn to_cubes(&self) -> Vec<Cube> {
        let mut cubes = Vec::with_capacity(256);
        for i in 0..(SIZE * SIZE * SIZE) {
            let block = self[i];
            let pos = index_to_pos(i, SIZE);
            cubes.append(
                &mut block
                    .to_cubes()
                    .iter_mut()
                    .map(|cube| {
                        cube.transform.x_axis.w += pos.x as f32;
                        cube.transform.y_axis.w += pos.y as f32;
                        cube.transform.z_axis.w += pos.z as f32;
                        *cube
                    })
                    .collect::<Vec<_>>(),
            );
        }
        cubes
    }
}
