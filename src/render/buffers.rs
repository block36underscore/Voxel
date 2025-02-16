use bevy::{
    math::Mat4,
    prelude::{
        Commands, FromWorld, Query, Res, ResMut, Resource, Transform, ViewVisibility, World,
    },
    render::{
        render_resource::{BufferUsages, BufferVec, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};
use bytemuck::{Pod, Zeroable};

use crate::world::chunk::ExtractedChunk;

use super::{
    pipeline::{create_bind_group, CubePullingPipeline, CubePullingShadowPipeline},
    PulledCube,
};

#[derive(Resource)]
pub(crate) struct PulledCubesBuffers {
    pub(crate) instances: BufferVec<Cube>,
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

pub(crate) fn update_buffers(
    mut buffers: ResMut<PulledCubesBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline: ResMut<CubePullingPipeline>,
    mut shadow_pipeline: ResMut<CubePullingShadowPipeline>,
    cubes: Query<(&PulledCube, &Transform, &ViewVisibility)>,
    chunks: Query<(&ExtractedChunk<16>, &Transform, &ViewVisibility)>,
) {
    buffers.instances.clear();

    for (_, transform, visibility) in &cubes {
        if !visibility.get() {
            continue;
        }

        buffers.instances.push(Cube {
            transform: transform.compute_matrix().transpose(),
        });
    }

    for (chunk, transform, visibility) in &chunks {
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

    buffers
        .instances
        .write_buffer(&render_device, &render_queue);

    pipeline.bind_group = create_bind_group(
        &render_device,
        &pipeline.layout,
        buffers.instances.buffer().unwrap(),
    );

    shadow_pipeline.bind_group = create_bind_group(
        &render_device,
        &pipeline.layout,
        buffers.instances.buffer().unwrap(),
    );
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

        PulledCubesBuffers { instances }
    }
}

pub(crate) fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<PulledCubesBuffers>();
}
