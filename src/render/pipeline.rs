use bevy::{asset::Handle, core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT}, ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, prelude::*, render::{render_phase::{BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases}, render_resource::{BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexState}, renderer::RenderDevice, view::{ExtractedView, RenderVisibleEntities, ViewUniform, ViewUniformOffset}}};

use super::{buffers::PulledCubesBuffers, PulledCube};



pub(crate) type DrawCustomPhaseItemCommands = (SetItemPipeline, DrawPulledCubesPhaseItem);
pub(crate) type WithCustomRenderedEntity = With<PulledCube>;

#[derive(Resource)]
pub(crate) struct CubePullingPipeline {
    pub(crate) shader: Handle<Shader>,
    pub(crate) layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
    pub(crate) view_layout: BindGroupLayout,
    pub(crate) view_bind_group: Option<BindGroup>,
}

impl SpecializedRenderPipeline for CubePullingPipeline {
    type Key = Msaa;

    fn specialize(&self, msaa: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![self.view_layout.clone(), self.layout.clone()],
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

        let view_layout_entries = vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::all(),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }];

        let view_layout = render_device.create_bind_group_layout(
            Some("view_layout"),
            &view_layout_entries,
        );

        CubePullingPipeline {
            shader: asset_server.load("shaders/vertex_pulled_cubes.wgsl"),
            layout,
            bind_group,
            view_layout,
            view_bind_group: None,
        }
    }
}



pub(crate) struct DrawPulledCubesPhaseItem;

pub(crate) fn queue_custom_phase_item(
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

pub(crate) fn create_bind_group(
    render_device: &RenderDevice, 
    layout: &BindGroupLayout, 
    buffer: &Buffer,
) -> BindGroup {
    render_device.create_bind_group(
        "cube_instance_data", 
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
        ]
    )
}

pub(crate) fn create_view_bind_group(
    render_device: &RenderDevice, 
    layout: &BindGroupLayout, 
    binding_resource: &BindingResource,
) -> BindGroup {
    render_device.create_bind_group(
        "view_uniform", 
        layout, 
        &[
            BindGroupEntry {
                binding: 0,
                resource: binding_resource.clone(), 
            }
        ]
    )
}

impl<P> RenderCommand<P> for DrawPulledCubesPhaseItem
where
    P: PhaseItem,
{
    type Param = (SRes<PulledCubesBuffers>, SRes<CubePullingPipeline>);

    type ViewQuery = Read<ViewUniformOffset>;

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        view_uniform_offset: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        resources: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let custom_phase_item_buffers = resources.0.into_inner();
        let pipeline = resources.1.into_inner();
        if let Some(view_bind_group) = &pipeline.view_bind_group {
            pass.set_bind_group(0, view_bind_group, &[view_uniform_offset.offset]);
        } else {
            panic!("No view bind group in vertex pulled cube pipeline")
        }

        pass.set_bind_group(1, 
            &pipeline.bind_group, &[]);

        // Draw one triangle (3 vertices).
        pass.draw(0..(custom_phase_item_buffers.instances.len() * 3) as u32, 0..1);

        RenderCommandResult::Success
    }
}





