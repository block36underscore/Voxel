use bevy::{asset::Handle, core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT}, ecs::{query::ROQueryItem, system::{lifetimeless::SRes, SystemParamItem}}, pbr::{ExtractedDirectionalLight, ExtractedPointLight, LightEntity, MeshPipeline, MeshPipelineKey, RenderCascadesVisibleEntities, RenderCubemapVisibleEntities, RenderVisibleMeshEntities, SetMeshViewBindGroup, SetPrepassViewBindGroup, Shadow, ShadowBinKey, ViewLightEntities}, prelude::*, render::{render_phase::{BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases}, render_resource::{BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState, RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexState}, renderer::RenderDevice, sync_world::MainEntity, view::ExtractedView}};

use super::{buffers::PulledCubesBuffers, PulledCube};

pub(crate) type DrawPulledCubesPrepassCommands = (
    SetItemPipeline, 
    SetPrepassViewBindGroup<0>,
    DrawPulledCubesPhaseItem,
);

pub(crate) type DrawPulledCubesCommands = (
    SetItemPipeline, 
    SetMeshViewBindGroup<0>,
    DrawPulledCubesPhaseItem,
);
pub(crate) type WithCustomRenderedEntity = With<PulledCube>;

#[derive(Resource)]
pub(crate) struct CubePullingPipeline {
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
                self.layout.clone()
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
        
        let mesh_pipeline = world.resource::<MeshPipeline>().clone();

        CubePullingPipeline {
            shader: asset_server.load("shaders/vertex_pulled_cubes.wgsl"),
            layout,
            bind_group,
            mesh_pipeline,
        }
    }
}

pub(crate) struct DrawPulledCubesPhaseItem;

pub(crate) struct DrawPulledCubesShadowPhaseItem;

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    pulled_cube_pipeline: Res<CubePullingPipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    mut shadow_render_phases: ResMut<ViewBinnedRenderPhases<Shadow>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    shadow_draw_functions: Res<DrawFunctions<Shadow>>,
    view_lights: Query<(Entity, &ViewLightEntities)>,
    view_light_entities: Query<&LightEntity>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<CubePullingPipeline>>,
    views: Query<(Entity, &Msaa), With<ExtractedView>>,
    point_light_entities: Query<&RenderCubemapVisibleEntities, With<ExtractedPointLight>>,
    directional_light_entities: Query<
        &RenderCascadesVisibleEntities,
        With<ExtractedDirectionalLight>,
    >,
    spot_light_entities: Query<
        &RenderVisibleMeshEntities, 
        With<ExtractedPointLight,>
    >,
) {
    let draw_pulled_cubes_phase_item = opaque_draw_functions
        .read()
        .id::<DrawPulledCubesCommands>();
    
    let pulled_cubes_prepass_phase_item = shadow_draw_functions
        .read()
        .id::<DrawPulledCubesPrepassCommands>();

    for (entity, view_lights) in &view_lights {
        for view_light_entity in view_lights.lights.iter().copied() {
            let Ok(light_entity) = 
                view_light_entities.get(view_light_entity) 
            else { continue; };
            
            let Some(shadow_phase) = 
                shadow_render_phases.get_mut(&view_light_entity) 
            else { continue; };
            
            let is_directional_light = matches!(
                light_entity, 
                LightEntity::Directional { .. }
            );
            let visible_entities = match light_entity {
                LightEntity::Directional {
                    light_entity,
                    cascade_index,
                } => directional_light_entities
                    .get(*light_entity)
                    .expect("Failed to get directional light visible entities")
                    .entities
                    .get(&entity)
                    .expect("Failed to get directional light visible entities for view")
                    .get(*cascade_index)
                    .expect("Failed to get directional light visible entities for cascade"),
                LightEntity::Point {
                    light_entity,
                    face_index,
                } => point_light_entities
                    .get(*light_entity)
                    .expect("Failed to get point light visible entities")
                    .get(*face_index),
                LightEntity::Spot { light_entity } => spot_light_entities
                    .get(*light_entity)
                    .expect("Failed to get spot light visible entities"),
            };
            let mut light_key = MeshPipelineKey::DEPTH_PREPASS;
            light_key.set(MeshPipelineKey::NONE, is_directional_light);

            // NOTE: Lights with shadow mapping disabled will have no visible entities
            // so no meshes will be queued

            if let LightEntity::Directional {
                    light_entity,
                    cascade_index,
                } = light_entity { directional_light_entities
                    .get(*light_entity)
                    .expect("Failed to get directional light visible entities")
                    .entities
                    .get(&entity)
                    .expect("Failed to get directional light visible entities for view")
                    .get(*cascade_index)
                    .expect("Failed to get directional light visible entities for cascade"); }
            
            for (entity, main_entity) in visible_entities.iter().copied() {
                let pipeline_id = specialized_render_pipelines.specialize(
                    &pipeline_cache,
                    &pulled_cube_pipeline,
                    MeshPipelineKey::NONE,
                );

                shadow_phase.add(
                    ShadowBinKey {
                        pipeline: pipeline_id,
                        draw_function: pulled_cubes_prepass_phase_item,
                        asset_id: AssetId::<Mesh>::invalid().untyped(),
                    },
                    (entity, main_entity),
                    BinnedRenderPhaseType::NonMesh,
                );
            }
        }
    }
    
    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view_entity, msaa) in views.iter() {
        
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };

        // Find all the custom rendered entities that are visible from this
        // view.
        println!("frame");
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
            opaque_phase.add(
                Opaque3dBinKey {
                    draw_function: draw_pulled_cubes_phase_item,
                    pipeline: pipeline_id,
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                    material_bind_group_id: None,
                    lightmap_image: None,
                },
                (Entity::PLACEHOLDER, MainEntity::from(Entity::PLACEHOLDER)),
                BinnedRenderPhaseType::NonMesh,
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

impl<P> RenderCommand<P> for DrawPulledCubesPhaseItem
where
    P: PhaseItem,
{
    type Param = (
        SRes<PulledCubesBuffers>, 
        SRes<CubePullingPipeline>, 
    );

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
        
        println!("render");
        
        pass.set_bind_group(1, 
            &pipeline.bind_group, &[]);

        // Draw one triangle (3 vertices).
        pass.draw(0..(custom_phase_item_buffers.instances.len() * 36) as u32, 0..1);

        RenderCommandResult::Success
    }
}

impl<P> RenderCommand<P> for DrawPulledCubesShadowPhaseItem
where
    P: PhaseItem,
{
    type Param = (
        SRes<PulledCubesBuffers>, 
        SRes<CubePullingPipeline>, 
    );

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
        
        pass.set_bind_group(1, 
            &pipeline.bind_group, &[]);

        // Draw one triangle (3 vertices).
        pass.draw(0..(custom_phase_item_buffers.instances.len() * 36) as u32, 0..1);

        RenderCommandResult::Success
    }
}

