use bevy::{app::{App, Plugin, PostUpdate}, core_pipeline::core_3d::Opaque3d, prelude::{Component, IntoSystemConfigs, Transform, ViewVisibility}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, view::{self, ViewUniforms, VisibilitySystems}, Render, RenderApp, RenderSet}};
use buffers::{prepare_custom_phase_item_buffers, update_buffers, PulledCubesBuffers};
use pipeline::{queue_custom_phase_item, CubePullingPipeline, DrawCustomPhaseItemCommands, WithCustomRenderedEntity};

pub mod buffers;
pub mod pipeline;

pub struct VoxelRendererPlugin;

impl Plugin for VoxelRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PulledCube>::default())
            .add_systems(
            PostUpdate,
            view::check_visibility::<WithCustomRenderedEntity>
                .in_set(VisibilitySystems::CheckVisibility),
        );

        app.get_sub_app_mut(RenderApp)
            .unwrap()
//            .init_resource::<CustomPhasePipeline>()
//            .init_resource::<SpecializedRenderPipelines<CustomPhasePipeline>>()
            .add_render_command::<Opaque3d, DrawCustomPhaseItemCommands>()
            .add_systems(
                Render,
                prepare_custom_phase_item_buffers.in_set(RenderSet::Prepare),
            )
            .add_systems(Render, (queue_custom_phase_item.in_set(RenderSet::Queue),
                update_buffers));

    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .expect("RenderApp does not exist")
            .init_resource::<PulledCubesBuffers>()
            .init_resource::<CubePullingPipeline>()
            .init_resource::<SpecializedRenderPipelines<CubePullingPipeline>>();
    }
}

#[derive(Clone, Component)]
#[require(Transform, ViewVisibility)]
pub struct PulledCube;

impl ExtractComponent for PulledCube {
    type QueryData = (&'static Self, &'static Transform, &'static ViewVisibility);
    type QueryFilter = ();
    type Out = (Self, Transform, ViewVisibility);

    fn extract_component(
        item: bevy::ecs::query::QueryItem<'_, Self::QueryData>
    ) -> Option<Self::Out> {
        let (marker, transform, visibility) = item;
        Some((marker.clone(), *transform, *visibility))
    }
}


