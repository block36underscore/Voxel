//! Demonstrates how to enqueue custom draw commands in a render phase.
//!
//! This example shows how to use the built-in
//! [`bevy_render::render_phase::BinnedRenderPhase`] functionality with a
//! custom [`RenderCommand`] to allow inserting arbitrary GPU drawing logic
//! into Bevy's pipeline. This is not the only way to add custom rendering code
//! into Bevy—render nodes are another, lower-level method—but it does allow
//! for better reuse of parts of Bevy's built-in mesh rendering logic.

use bevy::{
    core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT},
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    math::{vec3, Vec3A},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        primitives::Aabb,
        render_phase::{
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, BufferUsages, BufferVec, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState, RawBufferVec, RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexState
        },
        renderer::{RenderDevice, RenderQueue},
        view::{self, ExtractedView, RenderVisibleEntities, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

/// A marker component that represents an entity that is to be rendered using
/// our custom phase item.
///
/// Note the [`ExtractComponent`] trait implementation. This is necessary to
/// tell Bevy that this object should be pulled into the render world.
#[derive(Clone, Component)]
#[require(Transform, ViewVisibility)]
struct PulledCube;

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

/// Holds a reference to our shader.
///
/// This is loaded at app creation time.
#[derive(Resource)]
struct CubePullingPipeline {
    shader: Handle<Shader>,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

/// A [`RenderCommand`] that binds the vertex and index buffers and issues the
/// draw command for our custom phase item.
struct DrawPulledCubesPhaseItem;

impl<P> RenderCommand<P> for DrawPulledCubesPhaseItem
where
    P: PhaseItem,
{
    type Param = (SRes<PulledCubesBuffers>, SRes<CubePullingPipeline>);

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        resources: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let custom_phase_item_buffers = resources.0.into_inner();
        let pipeline = resources.1.into_inner();

        

        pass.set_bind_group(0, 
            &pipeline.bind_group, &[]);

        // Draw one triangle (3 vertices).
        pass.draw(0..(custom_phase_item_buffers.instances.len() * 3) as u32, 0..1);

        RenderCommandResult::Success
    }
}

/// The GPU vertex and index buffers for our custom phase item.
///
/// As the custom phase item is a single triangle, these are uploaded once and
/// then left alone.
#[derive(Resource)]
struct PulledCubesBuffers {
    instances: BufferVec<Triangle>,
}

fn update_buffers(
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
            Triangle {
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

#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
struct Triangle {
    transform: Mat4,
}

/// The custom draw commands that Bevy executes for each entity we enqueue into
/// the render phase.
type DrawCustomPhaseItemCommands = (SetItemPipeline, DrawPulledCubesPhaseItem);

/// A query filter that tells [`view::check_visibility`] about our custom
/// rendered entity.
type WithCustomRenderedEntity = With<PulledCube>;

const SCALE: f32 = 0.3;



/// The entry point.
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(ExtractComponentPlugin::<PulledCube>::default())
        .add_plugins(CustomRenderPlugin)
        .add_systems(Startup, setup)
        // Make sure to tell Bevy to check our entity for visibility. Bevy won't
        // do this by default, for efficiency reasons.
        .add_systems(
            PostUpdate,
            view::check_visibility::<WithCustomRenderedEntity>
                .in_set(VisibilitySystems::CheckVisibility),
        );

    // We make sure to add these to the render app, not the main app.
    app.get_sub_app_mut(RenderApp)
        .unwrap()
//        .init_resource::<CustomPhasePipeline>()
//        .init_resource::<SpecializedRenderPipelines<CustomPhasePipeline>>()
        .add_render_command::<Opaque3d, DrawCustomPhaseItemCommands>()
        .add_systems(
            Render,
            prepare_custom_phase_item_buffers.in_set(RenderSet::Prepare),
        )
        .add_systems(Render, (queue_custom_phase_item.in_set(RenderSet::Queue),
                update_buffers));

    app.run();
}

struct CustomRenderPlugin;

impl Plugin for CustomRenderPlugin {
    fn build(&self, _: &mut App) {}

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .expect("RenderApp does not exist")
            .init_resource::<PulledCubesBuffers>()
            .init_resource::<CubePullingPipeline>()
            .init_resource::<SpecializedRenderPipelines<CubePullingPipeline>>();
    }
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands) {
    // Spawn a single entity that has custom rendering. It'll be extracted into
    // the render world via [`ExtractComponent`].
    commands.spawn((
        Visibility::default(),
        Transform::from_translation(Vec3::new(0.1, 0.2, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        PulledCube,
    ));

    commands.spawn((
        Visibility::default(),
        Transform::from_translation(Vec3::new(-0.5, 0.2, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        PulledCube,
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Creates the [`CustomPhaseItemBuffers`] resource.
///
/// This must be done in a startup system because it needs the [`RenderDevice`]
/// and [`RenderQueue`] to exist, and they don't until [`App::run`] is called.
fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<PulledCubesBuffers>();
}

/// A render-world system that enqueues the entity with custom rendering into
/// the opaque render phases of each view.
fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    custom_phase_pipeline: Res<CubePullingPipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<CubePullingPipeline>>,
    views: Query<(Entity, &RenderVisibleEntities, &Msaa), With<ExtractedView>>,
) {
    let draw_custom_phase_item = opaque_draw_functions
        .read()
        .id::<DrawCustomPhaseItemCommands>();

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view_entity, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };

        // Find all the custom rendered entities that are visible from this
        // view.
        for &entity in view_visible_entities
            .get::<WithCustomRenderedEntity>()
            .iter()
        {
            // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
            // some per-view settings, such as whether the view is HDR, but for
            // simplicity's sake we simply hard-code the view's characteristics,
            // with the exception of number of MSAA samples.
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &custom_phase_pipeline,
                *msaa,
            );

            // Add the custom render item. We use the
            // [`BinnedRenderPhaseType::NonMesh`] type to skip the special
            // handling that Bevy has for meshes (preprocessing, indirect
            // draws, etc.)
            //
            // The asset ID is arbitrary; we simply use [`AssetId::invalid`],
            // but you can use anything you like. Note that the asset ID need
            // not be the ID of a [`Mesh`].
            opaque_phase.add(
                Opaque3dBinKey {
                    draw_function: draw_custom_phase_item,
                    pipeline: pipeline_id,
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                    material_bind_group_id: None,
                    lightmap_image: None,
                },
                entity,
                BinnedRenderPhaseType::NonMesh,
            );
        }
    }
}

impl SpecializedRenderPipeline for CubePullingPipeline {
    type Key = Msaa;

    fn specialize(&self, msaa: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![self.layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    // Ordinarily, you'd want to check whether the view has the
                    // HDR format and substitute the appropriate texture format
                    // here, but we omit that for simplicity.
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            // Note that if your view has no depth buffer this will need to be
            // changed.
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

impl FromWorld for PulledCubesBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();


        let mut instances = BufferVec::new(BufferUsages::STORAGE);

        instances.push(Triangle {transform: Mat4::ZERO});

        instances.write_buffer(render_device, render_queue);

        PulledCubesBuffers {
            instances,
        }
    }
}

fn create_bind_group(
    render_device: &RenderDevice, 
    layout: &BindGroupLayout, 
    buffer: &Buffer
) -> BindGroup {
    render_device.create_bind_group(
        "instance_data", 
        layout, 
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(
                    BufferBinding {
                        buffer,
                        offset: 0,
                        size: None,
                    }
                )
            }
        ])

}

impl FromWorld for CubePullingPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        let render_device = world.resource::<RenderDevice>();

        let buffers = world.resource::<PulledCubesBuffers>();

        let layout = render_device
            .create_bind_group_layout(
                None,
                &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            );

        let bind_group = create_bind_group(
            render_device, 
            &layout, 
            buffers.instances.buffer().unwrap()
        );
        
        CubePullingPipeline {
            shader: asset_server.load("shaders/vertex_pulled_cubes.wgsl"),
            layout,
            bind_group,
        }
    }
}

