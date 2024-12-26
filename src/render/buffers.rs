use bevy::{math::Mat4, pbr::{GpuLights, LightMeta, MeshPipeline}, prelude::{Commands, FromWorld, Query, Res, ResMut, Resource, Transform, ViewVisibility, World}, render::{render_resource::{BufferUsages, BufferVec, ShaderType}, renderer::{RenderDevice, RenderQueue}, view::ViewUniforms}};
use bytemuck::{Pod, Zeroable};

use super::{pipeline::{create_bind_group, CubePullingPipeline}, PulledCube};

#[derive(Resource)]
pub(crate) struct PulledCubesBuffers {
    pub(crate) instances: BufferVec<Cube>,
}

#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct Cube {
    pub transform: Mat4,
}

pub(crate) fn update_buffers(
    mut buffers: ResMut<PulledCubesBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline: ResMut<CubePullingPipeline>,
    entities: Query<(&PulledCube, &Transform, &ViewVisibility)>,
) {
    buffers.instances.clear();

    for (_, transform, visibility) in &entities {
        if !visibility.get() { continue; }

        buffers.instances.push(
            Cube {
                transform: transform.compute_matrix().transpose(),
            }
        );
   }

    buffers.instances.write_buffer(&render_device, &render_queue);

    pipeline.bind_group = create_bind_group(
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

        instances.push(Cube {transform: Mat4::ZERO});

        instances.write_buffer(render_device, render_queue);

        PulledCubesBuffers {
            instances,
        }
    }
}

pub(crate) fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<PulledCubesBuffers>();
}

