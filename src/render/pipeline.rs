
use bevy::{
    asset::Handle,
    core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT},
    ecs::{
        component::Tick, query::ROQueryItem, system::{lifetimeless::SRes, SystemParamItem}
    },
    pbr::{
        ExtractedDirectionalLight, ExtractedPointLight, LightEntity, MeshPipeline, MeshPipelineKey,
        RenderCascadesVisibleEntities, RenderCubemapVisibleEntities, RenderVisibleMeshEntities,
        SetMeshViewBindGroup, SetPrepassViewBindGroup, ViewLightEntities,
    },
    prelude::*,
    render::{
        globals::GlobalsUniform,
        render_phase::{
            BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutEntries, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
            BufferBinding, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
            DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexState,
        },
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::{ExtractedView, ViewUniform},
    },
};

use super::{buffers::{PulledCubesBufferArrays, PulledCubesBuffers}, PulledCube};

pub(crate) type DrawPulledCubesPrepassCommands = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    DrawPulledCubesShadowPhaseItem,
);

pub(crate) type DrawPulledCubesCommands = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    DrawPulledCubesPhaseItem,
);
pub(crate) type WithCustomRenderedEntity = With<PulledCube>;

#[derive(Resource)]
pub struct CubePullingPipeline {
    pub(crate) shader: Handle<Shader>,
    pub(crate) layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
    pub(crate) mesh_pipeline: MeshPipeline,
}

impl SpecializedRenderPipeline for CubePullingPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // let layout = &mesh_pipeline.get_view_layout(
        //     MeshPipelineViewLayoutKey::from(msaa)
        // );

        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![
                self.mesh_pipeline.get_view_layout(key.into()).clone(),
                self.layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec!["SHADOW_FILTER_METHOD_GAUSSIAN".into()],
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
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

impl FromWorld for CubePullingPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        let render_device = world.resource::<RenderDevice>();

        let buffers = world.resource::<PulledCubesBufferArrays>();

        let layout = buffers.chunks_layout.clone();

        let bind_group = buffers.as_bind_group(render_device);

        let mesh_pipeline = world.resource::<MeshPipeline>().clone();

        CubePullingPipeline {
            shader: asset_server.load("shaders/vertex_pulled_cubes.wgsl"),
            bind_group,
            layout,
            mesh_pipeline,
        }
    }
}

#[derive(Resource)]
pub struct CubePullingShadowPipeline {
    pub(crate) shader: Handle<Shader>,
    pub(crate) layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
    pub(crate) view_layout: BindGroupLayout,
}

impl SpecializedRenderPipeline for CubePullingShadowPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // let layout = &mesh_pipeline.get_view_layout(
        //     MeshPipelineViewLayoutKey::from(msaa)
        // );

        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![self.view_layout.clone(), self.layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "shadow_vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "shadow_fragment".into(),
                targets: vec![],
            }),
            primitive: PrimitiveState::default(),
            // Note that if your view has no depth buffer this will need to be
            // changed.
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

impl FromWorld for CubePullingShadowPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        let render_device = world.resource::<RenderDevice>();

        let buffers = world.resource::<PulledCubesBuffers>();

        let layout = render_device.create_bind_group_layout(
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

        let bind_group =
            create_bind_group(render_device, &layout, buffers.instances.buffer().unwrap());

        let view_layout = render_device.create_bind_group_layout(
            "prepass_view_layout_no_motion_vectors",
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    (0, uniform_buffer::<ViewUniform>(true)),
                    (1, uniform_buffer::<GlobalsUniform>(false)),
                ),
            ),
        );

        CubePullingShadowPipeline {
            shader: asset_server.load("shaders/vertex_pulled_cubes.wgsl"),
            layout,
            bind_group,
            view_layout,
        }
    }
}

pub(crate) struct DrawPulledCubesPhaseItem;

pub(crate) struct DrawPulledCubesShadowPhaseItem;

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    pulled_cube_pipeline: Res<CubePullingPipeline>,
//    pulled_cube_shadow_pipeline: Res<CubePullingShadowPipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
//    mut shadow_render_phases: ResMut<ViewBinnedRenderPhases<Shadow>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
//    shadow_draw_functions: Res<DrawFunctions<Shadow>>,
    view_lights: Query<(Entity, &ViewLightEntities)>,
    view_light_entities: Query<&LightEntity>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<CubePullingPipeline>>,
//    mut specialized_shadow_render_pipelines: ResMut<
//        SpecializedRenderPipelines<CubePullingShadowPipeline>,
//    >,
    views: Query<(&ExtractedView, &Msaa)>,
    _point_light_entities: Query<&RenderCubemapVisibleEntities, With<ExtractedPointLight>>,
    directional_light_entities: Query<
        &RenderCascadesVisibleEntities,
        With<ExtractedDirectionalLight>,
    >,
    _spot_light_entities: Query<&RenderVisibleMeshEntities, With<ExtractedPointLight>>,
    mut next_tick: Local<Tick>,
) {
    let draw_pulled_cubes_phase_item = opaque_draw_functions.read().id::<DrawPulledCubesCommands>();

//    let pulled_cubes_prepass_phase_item = shadow_draw_functions
//        .read()
//        .id::<DrawPulledCubesPrepassCommands>();

    for (entity, view_lights) in &view_lights {
        for view_light_entity in view_lights.lights.iter().copied() {
            let Ok(light_entity) = view_light_entities.get(view_light_entity) else {
                continue;
            };

//            let Some(shadow_phase) = shadow_render_phases.get_mut(&view_light_entity) else {
//                continue;
//            };

            let is_directional_light = matches!(light_entity, LightEntity::Directional { .. });
            let mut light_key = MeshPipelineKey::DEPTH_PREPASS;
            light_key.set(MeshPipelineKey::NONE, is_directional_light);

            // NOTE: Lights with shadow mapping disabled will have no visible entities
            // so no meshes will be queued

            if let LightEntity::Directional {
                light_entity,
                cascade_index,
            } = light_entity
            {
                directional_light_entities
                    .get(*light_entity)
                    .expect("Failed to get directional light visible entities")
                    .entities
                    .get(&entity)
                    .expect("Failed to get directional light visible entities for view")
                    .get(*cascade_index)
                    .expect("Failed to get directional light visible entities for cascade");
            }

//            let pipeline_id = specialized_shadow_render_pipelines.specialize(
//                &pipeline_cache,
//                &pulled_cube_shadow_pipeline,
//                MeshPipelineKey::NONE,
//            );

//            shadow_phase.add(
//                ShadowBinKey {
//                    pipeline: pipeline_id,
//                    draw_function: pulled_cubes_prepass_phase_item,
//                    asset_id: AssetId::<Mesh>::invalid().untyped(),
//                },
//                (Entity::PLACEHOLDER, MainEntity::from(Entity::PLACEHOLDER)),
//                BinnedRenderPhaseType::NonMesh,
//            );
        }
    }

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };


        // Find all the custom rendered entities that are visible from this
        // view.
        // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
        // some per-view settings, such as whether the view is HDR, but for
        // simplicity's sake we simply hard-code the view's characteristics,
        // with the exception of number of MSAA samples.
        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        let pipeline_id = specialized_render_pipelines.specialize(
            &pipeline_cache,
            &pulled_cube_pipeline,
            view_key,
        );

        // Add the custom render item. We use the
        // [`BinnedRenderPhaseType::NonMesh`] type to skip the special
        // handling that Bevy has for meshes (preprocessing, indirect
        // draws, etc.)
        //
        // The asset ID is arbitrary; we simply use [`AssetId::invalid`],
        // but you can use anything you like. Note that the asset ID need
        // not be the ID of a [`Mesh`].

        let this_tick = next_tick.get() + 1;
        next_tick.set(this_tick);

        opaque_phase.add(
            Opaque3dBatchSetKey {
                draw_function: draw_pulled_cubes_phase_item,
                pipeline: pipeline_id,
                material_bind_group_index: None,
                lightmap_slab: None,
                vertex_slab: default(),
                index_slab: None,
            },
            Opaque3dBinKey {
                asset_id: AssetId::<Mesh>::invalid().untyped(),
            },
            (Entity::PLACEHOLDER, MainEntity::from(Entity::PLACEHOLDER)),
            InputUniformIndex::default(),
            BinnedRenderPhaseType::NonMesh,
            *next_tick,
        );
    }
}

pub(crate) fn create_bind_group(
    render_device: &RenderDevice,
    layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    render_device.create_bind_group(
        "cube_instance_data",
        layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer,
                offset: 0,
                size: None,
            }),
        }],
    )
}

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

        pass.set_bind_group(1, &pipeline.bind_group, &[]);

        pass.draw(
            0..(3 * 2 * 3), 0..1
        );
        //pass.draw(
        //    0..(custom_phase_item_buffers.instances.len() as u32 * VERTICES_PER_CUBE),
        //    0..1,
        //);

        RenderCommandResult::Success
    }
}

const VERTICES_PER_CUBE : u32 = 18;

impl<P> RenderCommand<P> for DrawPulledCubesShadowPhaseItem
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

        pass.set_bind_group(1, &pipeline.bind_group, &[]);

        // Draw one triangle (3 vertices).
        pass.draw(
            0..(custom_phase_item_buffers.instances.len() as u32 * VERTICES_PER_CUBE),
            0..1,
        );

        RenderCommandResult::Success
    }
}
