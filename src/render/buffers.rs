use std::{num::NonZero, usize};

use bevy::{
    math::{Mat4, Vec3}, prelude::{
        Commands, FromWorld, Query, Res, ResMut, Resource, Transform, ViewVisibility, World,
    }, render::{
        render_resource::{BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages, BufferVec, ShaderStages, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    }
};
use bytemuck::{Pod, Zeroable};

use crate::world::chunk::ExtractedChunk;

use super::{
    pipeline::{create_bind_group, CubePullingPipeline},
    PulledCube,
};

pub const MAX_CHUNK_COUNT : u32 = 256;

#[derive(Resource)]
pub struct PulledCubesBuffers {
    pub(crate) instances: BufferVec<Cube>,
    pub(crate) dirty: bool,
}

#[derive(Resource)]
pub struct PulledCubesBufferArrays {
    pub chunk_instances: Vec<BufferVec<Cube>>,
    pub chunks_layout: BindGroupLayout,
}

impl FromWorld for PulledCubesBufferArrays {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut instances1 = BufferVec::new(BufferUsages::STORAGE);

        instances1.push(Cube {
            transform: Mat4::ZERO,
        });

        let mut instances2 = BufferVec::new(BufferUsages::STORAGE);

        instances2.push(Cube {
            transform: Mat4::from_translation(Vec3::new(1., 3., 1.)),
        });

        instances2.push(Cube {
            transform: Mat4::from_translation(Vec3::new(5., 3., 1.)),
        });

        instances1.write_buffer(render_device, render_queue);
        instances2.write_buffer(render_device, render_queue);

        let chunk_instances = vec![
            instances1,
            instances2,
        ];

        let chunks_layout = render_device.create_bind_group_layout(
            "chunk_arrays_buffer_layout",
            Self::bind_group_layout_entries().as_slice(),
        );

        Self {
            chunk_instances,
            chunks_layout,
        }
    }
}

impl PulledCubesBufferArrays {

    pub fn as_bind_group(
            &self,
            render_device: &RenderDevice,
        ) -> BindGroup {
        let handles = self
            .chunk_instances
            .iter()
            .take(MAX_CHUNK_COUNT as usize)
            .map(|buffer| buffer
                    .buffer()
                    .unwrap()
                    .as_entire_buffer_binding())
            .collect::<Vec<_>>();

        render_device.create_bind_group(
            "main_chunk_buffers",
            &self.chunks_layout,
            &BindGroupEntries::single(BindingResource::BufferArray(&handles[..]))
        )
    }

    pub fn bind_group_layout_entries()
        -> Vec<BindGroupLayoutEntry>
        where Self: Sized {
        vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: true,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: NonZero::new(MAX_CHUNK_COUNT),
            },
        ]
    }
}

#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct Cube {
    pub transform: Mat4,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
        }
    }
}

pub trait ToCubes {
    fn to_cubes(&self) -> Vec<Cube>;
}

pub fn update_buffers(
    mut buffers: ResMut<PulledCubesBuffers>,
    cubes: Query<(&PulledCube, &Transform, &ViewVisibility)>,
    chunks: Query<(&ExtractedChunk<16>, &Transform, &ViewVisibility)>,
) {
    if !buffers.dirty {
        return;
    }
    
    buffers.dirty = true;

    buffers.instances.clear();

    for (_, transform, visibility) in &cubes {
        if !visibility.get() {
            continue;
        }

        buffers.instances.push(Cube {
            transform: transform.compute_matrix().transpose(),
        });
    }

    for (chunk, transform, _visibility) in &chunks {
        // if !visibility.get() {
        //     continue;
        // }

        let matrix = transform.compute_matrix().transpose();

        chunk
            .to_cubes()
            .iter_mut()
            .map(|cube| {
                cube.transform *= matrix;
                cube
            })
            .for_each(|cube| {
                buffers.instances.push(*cube);
            });
    }
}

pub fn update_chunk_buffers<const SIZE: usize>(
    mut buffers: ResMut<PulledCubesBuffers>,
    chunks: Query<(&ExtractedChunk<SIZE>, &Transform, &ViewVisibility)>,
) {
    if !buffers.dirty {
        return;
    }
    
    buffers.dirty = false;

    for (chunk, transform, _) in &chunks {
        // if !visibility.get() {
        //     continue;
        // }

        let matrix = transform.compute_matrix().transpose();

        chunk
            .to_cubes()
            .iter_mut()
            .map(|cube| {
                cube.transform *= matrix;
                cube
            })
            .for_each(|cube| {
                buffers.instances.push(*cube);
            });
    }
}

pub fn write_buffers(
    mut buffers: ResMut<PulledCubesBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline: ResMut<CubePullingPipeline>,
//    mut shadow_pipeline: ResMut<CubePullingShadowPipeline>,
) {
    buffers
        .instances
        .write_buffer(&render_device, &render_queue);


    pipeline.bind_group = create_bind_group(
            &render_device,
            &pipeline.layout,
            buffers.instances.buffer().unwrap(),
        );

//        shadow_pipeline.bind_group = create_bind_group(
//            &render_device,
//            &pipeline.layout,
//            buffers.instances.buffer().unwrap(),
//        );
}

impl FromWorld for PulledCubesBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut instances = BufferVec::new(BufferUsages::STORAGE);

        instances.push(Cube {
            transform: Mat4::ZERO,
        });

        instances.write_buffer(render_device, render_queue);

        PulledCubesBuffers {
            instances,
            dirty: true,
        }
    }
}

pub(crate) fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<PulledCubesBuffers>();
}
