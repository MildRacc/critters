use std::borrow::Cow;

use bevy::{app::{App, Startup, Update}, asset::{Asset, AssetServer, Assets, Handle}, core_pipeline::core_2d::graph::Node2d, ecs::{message::MessageReader, observer::On, query::With, resource::Resource, schedule::{IntoScheduleConfigs, common_conditions::{not, resource_exists}}, system::{Commands, Query, Res, ResMut, Single}}, input::{ButtonInput, mouse::{AccumulatedMouseMotion, MouseButton, MouseWheel}}, math::{Vec2, Vec3, VectorSpace, primitives::Rectangle}, mesh::{Mesh, Mesh2d}, post_process::motion_blur::pipeline, reflect::TypePath, render::{Render, RenderApp, RenderStartup, RenderSystems, extract_resource::ExtractResource, gpu_readback::{Readback, ReadbackComplete}, render_asset::RenderAssets, render_graph::{NodeRunError, RenderGraph, RenderGraphContext, RenderLabel}, render_resource::{AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, BufferUsages, CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, UniformBuffer, binding_types::{storage_buffer, uniform_buffer}}, renderer::{RenderContext, RenderDevice, RenderQueue}, storage::{GpuShaderStorageBuffer, ShaderStorageBuffer}}, sprite_render::{Material2d, MeshMaterial2d}, time::Time, transform::components::Transform, window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window}};
use rand::{RngExt, rng};

use crate::simulation::{self, critter::{self, PhysicsCritter, ShaderCritter}};



const CRITTER_COUNT: usize = 6400;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CritterComputeLabel;
pub struct ComputePlugin;
impl bevy::app::Plugin for ComputePlugin
{
    fn build(&self, app: &mut App)
    {

        app.add_systems(Update, sync_material_buffer);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else 
        {
            return;
        };

        // Render
        render_app.add_systems(RenderStartup, init_critter_pipeline);
        render_app.add_systems(Render, prepare_bind_group.in_set(RenderSystems::PrepareBindGroups));
        render_app.add_systems(Render, update.in_set(RenderSystems::Prepare));


        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(
            CritterComputeLabel,
            CritterComputeNode
        );

    }
}



#[derive(Resource)]
pub struct World
{
    pub resolution: Vec2,

    pub camera_position: Vec2,
    pub camera_zoom: f32,
    pub camera_sensitivity: f32,

    pub material_handle: Handle<VisMaterial>,
    pub viewport: Handle<Mesh>,

    pub drag_coefficient: f32
}



#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct VisMaterial
{
    #[storage(0, read_only)]
    pub critters: Handle<ShaderStorageBuffer>,
    #[uniform(1)]
    pub critter_count: u32,
    #[uniform(2)]
    pub resolution: Vec2,
    #[uniform(3)]
    pub cam_offset: Vec2
}
impl Material2d for VisMaterial
{
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shader/critter.wgsl".into()
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub enum ShaderState
{
    Update(bool)
}



#[derive(Resource)]
pub struct CritterSimPipeline
{
    bind_group_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId
}


#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer
{
    buffer_a: Handle<ShaderStorageBuffer>,
    buffer_b: Handle<ShaderStorageBuffer>
}

#[derive(Resource)]
pub struct GpuBufferBindGroup([BindGroup; 2]);



pub fn setup(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>
)
{

    commands.insert_resource(ShaderState::Update(false));

    let mut buf_a: Vec<simulation::critter::GpuCritter> = Vec::new();

    let mut rng = rand::rng();

    for _ in 0..CRITTER_COUNT
    {
        let (rx, ry) = (rng.random_range(-1_000.0..1_000.0), rng.random_range(-1_000..1_000));
        
        buf_a.push(simulation::critter::GpuCritter {
            position: Vec2::new(rx as f32, ry as f32),
            velocity: Vec2::ZERO,
            radius: 16.0,
            color: rng.random_range(0..8),
            _padding: Vec2::ZERO
        });

    }

    let buf_b = buf_a.clone();


    let mut buf_a = ShaderStorageBuffer::from(buf_a);
    let mut buf_b = ShaderStorageBuffer::from(buf_b);

    buf_a.buffer_description.usage |= BufferUsages::COPY_SRC;
    buf_a.buffer_description.usage |= BufferUsages::STORAGE;
    buf_b.buffer_description.usage |= BufferUsages::COPY_SRC;
    buf_b.buffer_description.usage |= BufferUsages::STORAGE;

    let buf_a = buffers.add(buf_a);
    let buf_b = buffers.add(buf_b);

    
    commands.insert_resource(ReadbackBuffer{buffer_a: buf_a, buffer_b: buf_b});


    let uniforms = simulation::critter::CritterUniforms
    {
        dt: 0.0,
        damping: 0.0625,
        critter_count: CRITTER_COUNT as u32
    };

    commands.insert_resource(uniforms);

}



pub fn init_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VisMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
    readback: Res<ReadbackBuffer>,
)
{

    let (w, h) = (window.resolution.width(), window.resolution.height());


    let material_handle = materials.add(VisMaterial
    {
        critters: readback.buffer_a.clone(),
        critter_count: CRITTER_COUNT as u32,
        resolution: Vec2::new(w, h),
        cam_offset: Vec2::ZERO
    });


    let viewport = meshes.add(Rectangle::new(w, h));

    commands.spawn((
        Mesh2d(viewport.clone()),
        MeshMaterial2d(material_handle.clone())
    ));

    commands.insert_resource(World {
        resolution: Vec2::new(window.width(), window.height()),
        
        camera_position: Vec2::new(0.0, 0.0),
        camera_zoom: 1.0,
        camera_sensitivity: 1.0,

        material_handle: material_handle,
        viewport: viewport,

        drag_coefficient: 0.025,
    });
}



pub fn prepare_bind_group(
    mut commands: Commands,
    buffer: Res<ReadbackBuffer>,
    buffers: ResMut<RenderAssets<GpuShaderStorageBuffer>>,
    pipeline: Res<CritterSimPipeline>,
    uniforms: Res<simulation::critter::CritterUniforms>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>
)
{

    let storage_a = buffers.get(&buffer.buffer_a).unwrap();
    let storage_b = buffers.get(&buffer.buffer_b).unwrap();

    let mut uniform_buffer = UniformBuffer::from(uniforms.into_inner());
    uniform_buffer.write_buffer(&render_device, &queue);

    let bind_group_0 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            storage_a.buffer.as_entire_binding(),
            storage_b.buffer.as_entire_binding(),
            &uniform_buffer
        ))
    );


    let bind_group_1 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            storage_b.buffer.as_entire_binding(),
            storage_a.buffer.as_entire_binding(),
            &uniform_buffer
        ))
    );

    commands.insert_resource(GpuBufferBindGroup([bind_group_0, bind_group_1]));

}



pub fn init_critter_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>
)
{
    let bind_group_layout = BindGroupLayoutDescriptor::new
    (
        "Critters",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<critter::GpuCritter>(false),
                storage_buffer::<critter::GpuCritter>(false),
                uniform_buffer::<critter::CritterUniforms>(false)
            )
        )
    );

    let shader = asset_server.load("shader/critter.wgsl");

    let compute_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor
    {
        layout: vec![bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("physics_compute")),
        ..Default::default()
    });

    commands.insert_resource(CritterSimPipeline
    {
        bind_group_layout: bind_group_layout,
        pipeline: compute_pipeline
    });

}


pub struct CritterComputeNode;
impl bevy::render::render_graph::Node for CritterComputeNode
{

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &bevy::prelude::World
    ) -> Result<(), NodeRunError>
    {

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<CritterSimPipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();
        let state = world.resource::<ShaderState>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline)
        {
            let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor
            {
                label: Some("GPU critter readback compute pass"),
                ..Default::default()
            });

            let index = match *state {ShaderState::Update(b) => b as usize};
            pass.set_bind_group(0, &bind_group.0[index], &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups((CRITTER_COUNT / 64) as u32, 1, 1);

        }

        Ok(())

    }

}


fn sync_material_buffer(
    mut materials: ResMut<Assets<VisMaterial>>,
    state: Res<ShaderState>,
    readback: Res<ReadbackBuffer>,
    world: Res<World>
)
{


    if let Some(mat) = materials.get_mut(&world.material_handle)
    {
        let index = match *state {ShaderState::Update(b) => b as usize};

        mat.critters = if index == 0 {
            readback.buffer_b.clone()
        } else {
            readback.buffer_a.clone()
        };

        mat.critter_count = CRITTER_COUNT as u32;
        mat.resolution = world.resolution * world.camera_zoom;
        mat.cam_offset = world.camera_position;
    }


}



pub fn update(
    mut state: ResMut<ShaderState>,
    mut uniforms: ResMut<simulation::critter::CritterUniforms>,
    time: Res<Time>
)
{

    match *state
    {
        ShaderState::Update(true) =>
        {
            *state = ShaderState::Update(false);
        }
        
        _ =>
        {
            *state = ShaderState::Update(true);
        }
    }

    uniforms.dt = time.delta_secs();

}


pub fn update_world(
    mut materials: ResMut<Assets<VisMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    world: ResMut<World>,
    phys_critters: Query<(&PhysicsCritter, &Transform)>
)
{

    let points: Vec<ShaderCritter> = phys_critters
        .iter()
        .map(|(c, t)| {
            ShaderCritter { 
                center: t.translation.truncate(), 
                color: c.color, 
                radius: c.radius, 
                _padding: Vec3::ZERO }
        } )
        .collect();



        if let Some(mat) = materials.get_mut(&world.material_handle)
        {

            mat.critter_count = points.len() as u32;
            mat.resolution = world.resolution * world.camera_zoom;
            mat.cam_offset = world.camera_position;
        }

}



pub fn update_resolution(
    mut world: ResMut<World>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Single<&Window, With<PrimaryWindow>>
)
{
    let (win_width, win_height) = (window.resolution.width(), window.resolution.height());
    let (world_width, world_height) = (world.resolution.x, world.resolution.y);




    if (win_width != world_width) || (win_height != world_height)
    {

        let new = Vec2::new(window.resolution.width(), window.resolution.height());

        if let Some(mesh) = meshes.get_mut(&world.viewport)
        {
            *mesh = Rectangle::new(new.x, new.y).into();
        }

        world.resolution = new
    }

}



pub fn player_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut cursor_options: Single<&mut CursorOptions>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut world: ResMut<World>
)
{

    if !window.focused {return;}


    if !mouse_input.pressed(MouseButton::Right)
    {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
        return;
    }
    else
    {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Confined;
    }

    let display_sens = (window.physical_width() / window.physical_height()) as f32 / 2.0;

    world.camera_position.x -= mouse_motion.delta.x * display_sens * world.camera_sensitivity;
    world.camera_position.y -= mouse_motion.delta.y * display_sens * world.camera_sensitivity;

    for scroll in mouse_wheel.read()
    {
        const TWO: f32 = 2.0;

        let scale = TWO.powf(-scroll.y / 4.0);

        world.camera_zoom *= scale;
        world.camera_sensitivity *= scale;

        if world.camera_zoom < 0.03125 {world.camera_zoom = 0.03125; world.camera_sensitivity = 0.03125}
        if world.camera_zoom > 32.0 {world.camera_zoom = 32.0; world.camera_sensitivity = 32.0}
    }

}
