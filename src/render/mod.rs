use bevy::{
    app::{App, Plugin},
    core_pipeline::core_3d::Opaque3d,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, renderer::RenderDevice, settings::WgpuFeatures, Render, RenderApp, RenderSet
    },
};
use buffers::{prepare_custom_phase_item_buffers, update_buffers, write_buffers, PulledCubesBuffers, PulledCubesBufferArrays};
use pipeline::{
    queue_custom_phase_item, CubePullingPipeline,
    DrawPulledCubesCommands,
};

use crate::world::chunk::ChunkPlugin;

pub mod buffers;
pub mod pipeline;

pub struct VoxelRendererPlugin;

impl Plugin for VoxelRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PulledCube>::default(),
            ChunkPlugin::<16>,
            GpuFeatureSupportChecker,
        ));

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Opaque3d, DrawPulledCubesCommands>()
//            .add_render_command::<Shadow, DrawPulledCubesPrepassCommands>()
            .add_systems(
                Render,
                prepare_custom_phase_item_buffers.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                (
                    queue_custom_phase_item.in_set(RenderSet::Queue),
                    update_buffers,
                    write_buffers,
                    ).chain(),
            );
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .expect("RenderApp does not exist")
            .init_resource::<PulledCubesBufferArrays>()
            .init_resource::<PulledCubesBuffers>()
            .init_resource::<CubePullingPipeline>()
//            .init_resource::<CubePullingShadowPipeline>()
            .init_resource::<SpecializedRenderPipelines<CubePullingPipeline>>();
//            .init_resource::<SpecializedRenderPipelines<CubePullingShadowPipeline>>();
    }
}


struct GpuFeatureSupportChecker;

impl Plugin for GpuFeatureSupportChecker {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world().resource::<RenderDevice>();

        // Check if the device support the required feature. If not, exit the example.
        // In a real application, you should setup a fallback for the missing feature
        if !render_device
            .features()
            .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
        {
            panic!(
"Render device doesn't support feature \
SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
which is required for texture binding arrays"
            );
        }
    }
}

#[derive(Clone, Component)]
#[require(Transform)]
pub struct PulledCube;

impl ExtractComponent for PulledCube {
    type QueryData = (&'static Self, &'static Transform, &'static ViewVisibility);
    type QueryFilter = ();
    type Out = (Self, Transform, ViewVisibility);

    fn extract_component(
        item: bevy::ecs::query::QueryItem<'_, Self::QueryData>,
    ) -> Option<Self::Out> {
        let (marker, transform, visibility) = item;
        Some((marker.clone(), *transform, *visibility))
    }
}
