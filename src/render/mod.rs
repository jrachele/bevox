use std::borrow::Cow;
use std::sync::Arc;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{RenderGraph, self};
use bevy::render::render_resource::{StorageBuffer, ShaderType, UniformBuffer, BindGroup, BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BufferBindingType, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BindGroupEntry, BindGroupDescriptor, BufferBinding, BindingResource, ComputePassDescriptor, Extent3d, TextureDimension, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue, RenderContext};
use bevy::render::{RenderApp, RenderSet};
use bevy::render::extract_resource::{ExtractResourcePlugin, ExtractResource};

use rand::Rng;

use crate::util::flycam::FlyCam;
use crate::util::*;
use crate::voxel::VoxelGrid;

#[derive(Resource, Default, Clone, ShaderType, ExtractResource)]
struct PlayerData {
    camera_matrix: Mat4,
    inverse_perspective_matrix: Mat4,
    mouse_click: u32,
    brush_size: u32,
}

#[derive(Resource, Clone, ExtractResource)]
struct VoxelGridStorage(Arc<StorageBuffer<VoxelGrid>>);

#[derive(Resource, Clone, ExtractResource)]
struct VoxelGridStorageDouble(Arc<StorageBuffer<VoxelGrid>>);

#[derive(Resource)]
struct PlayerDataUniform(UniformBuffer<PlayerData>);

#[derive(Resource, Clone, ExtractResource)]
struct PhysicsTimer {
    elapsed_time: f32,
    trigger_time: f32,
}

impl PhysicsTimer {
    fn triggered(&self) -> bool {
        self.elapsed_time >= self.trigger_time
    }
    fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }
    fn tick(&mut self, amount: f32) {
        self.elapsed_time += amount;
    }
}

#[derive(Resource, Clone, Deref, ExtractResource)]
struct RaycastOutputImage(Handle<Image>);

// Bind groups
#[derive(Resource)]
struct PhysicsUniformBindGroup(BindGroup);

#[derive(Resource)]
struct VoxelGridStorageBindGroup(BindGroup);

#[derive(Resource)]
struct VoxelGridStorageDoubleBindGroup(BindGroup);

#[derive(Resource)]
struct RaycastImageBindGroup(BindGroup);

#[derive(Resource)]
pub struct ComputePipeline {
    voxel_data_bind_group_layout: BindGroupLayout,
    physics_data_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    compute_physics: CachedComputePipelineId,
    compute_buffer_swap: CachedComputePipelineId,
    compute_raycast: CachedComputePipelineId,
}

pub struct RenderComputePlugin;

impl Plugin for RenderComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<VoxelGridStorage>::default());
        app.add_plugin(ExtractResourcePlugin::<VoxelGridStorageDouble>::default());
        app.add_plugin(ExtractResourcePlugin::<PlayerData>::default());
        app.add_plugin(ExtractResourcePlugin::<PhysicsTimer>::default());
        app.add_plugin(ExtractResourcePlugin::<RaycastOutputImage>::default());

        app.add_startup_system(setup);
        app.add_system(update_player_uniform);
        app.add_system(update_physics_timer);
        // app.register_type::<VoxelGrid>();
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputePipeline>()
            .add_system(write_uniform_buffers.in_set(RenderSet::Prepare))
            // .add_system(update_physics_timer.in_set(RenderSet::Prepare))
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raycast", RayCastRenderNode::default());
        render_graph.add_node_edge("raycast", bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}
const SCREEN_SIZE: (u32, u32) = (1920, 1080);
const VOXEL_GRID_SIZE: u32 = 128u32;
const WORKGROUP_SIZE: u32 = 8;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {

    let pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    let n = VOXEL_GRID_SIZE;
    let r = (n - 1) as f32;
    let mut voxels = VoxelGrid::new(n, pos);

    let mut rng = rand::thread_rng();

    // Create a sphere for testing purposes
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let pos = (Vec3::new(i as f32, j as f32, k as f32) * 2.0) - Vec3::splat((n) as f32);
                let in_sphere = pos.length_squared() <= r * r;
                if let Some(mut voxel) = voxels.get_mut(i, j, k) {
                    // if in_sphere {
                        // For now just use sand and add nice variance
                        // TODO: Add model loading
                        // let is_sand = rng.gen_bool(0.5);

                        // if is_sand {
                            let variance = rng.gen_range(-0.02..0.02);
                            let sand_color = Vec3::new(0.5, 0.3,  0.1);
                            let varied_sand = vary_color(sand_color, variance);
                            voxel.set_color(varied_sand);
                        // }
                        // else {
                            // let water_color = Vec3::new(0.3, 0.7, 0.9);
                            // voxel.set_color(water_color);
                            // voxel.set_voxel_type(1);
                        // }
                    // }
                }
            }
        }
    }

    // Create a storage buffer containing our voxel data
    let mut buffer = StorageBuffer::<VoxelGrid>::from(voxels.clone());
    buffer.write_buffer(&render_device, &render_queue);

    commands.insert_resource(VoxelGridStorage(Arc::new(buffer)));

    {
        // Create a double buffer for voxel data, for cellular automata
        let mut buffer = StorageBuffer::<VoxelGrid>::from(voxels);
        buffer.write_buffer(&render_device, &render_queue);

        commands.insert_resource(VoxelGridStorageDouble(Arc::new(buffer)));
    }

    // Create a uniform buffer for dynamic data like camera position, brush size, and mouse clicking
    let uniform = PlayerData::default();
    commands.insert_resource(uniform);

    // Set up a timer to compute physics at a fixed interval
    let physics_timer = PhysicsTimer {
        elapsed_time: 0.0,
        trigger_time: 1.0 / 30.0
    };
    commands.insert_resource(physics_timer);

    // Create the 2D texture buffer to render the results of the raycast
    let mut image = Image::new_fill(
        Extent3d {
            width: SCREEN_SIZE.0,
            height: SCREEN_SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32)),

            ..default()
        },
        texture: image.clone(),

        ..default()
    });

    commands.insert_resource(RaycastOutputImage(image));

    commands.spawn(Camera2dBundle::default());
}
fn create_perspective_projection_matrix(aspect_ratio : f32, fov : f32, near : f32, far : f32) -> Mat4 {
    let tan_half_fov = f32::tan(fov * 0.5 * 3.14159265 / 180.0);
    let sx = 1.0 / (aspect_ratio * tan_half_fov);
    let sy = 1.0 / tan_half_fov;
    let sz = -(far + near) / (far - near);
    let pz = -(2.0 * far * near) / (far - near);
    return Mat4::from_cols(
        Vec4::new(sx, 0.0, 0.0, 0.0),
        Vec4::new(0.0, sy, 0.0, 0.0),
        Vec4::new(0.0, 0.0, sz, -1.0),
        Vec4::new(0.0, 0.0, pz, 0.0),
    );
}

fn update_player_uniform(
    mut uniform_data: ResMut<PlayerData>,
    transform_query: Query<&Transform, With<FlyCam>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if let Ok(transform) = transform_query.get_single() {
        uniform_data.camera_matrix = transform.compute_matrix();
        let perspective_matrix = create_perspective_projection_matrix(16.0 / 9.0, 60.0, 0.1, 1000.0);
        uniform_data.inverse_perspective_matrix = perspective_matrix.inverse();
    }
    let mut mouse_buttons = 0u32;
    if mouse_input.pressed(MouseButton::Left) {
        mouse_buttons |= 1;
    }
    if mouse_input.pressed(MouseButton::Middle) {
        mouse_buttons |= 1 << 1;
    }
    if mouse_input.pressed(MouseButton::Right) {
        mouse_buttons |= 1 << 2;
    }
    uniform_data.mouse_click = mouse_buttons;

    uniform_data.brush_size = 3;

}

fn update_physics_timer(
    mut physics_timer: ResMut<PhysicsTimer>,
    time: Res<Time>
) {
    // If the timer was triggered, reset it
    if physics_timer.triggered() {
        physics_timer.reset();
    }

    physics_timer.tick(time.delta_seconds());
}

fn write_uniform_buffers(
    mut commands: Commands,
    camera_data: ResMut<PlayerData>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // player data
    {
        let mut buffer = UniformBuffer::<PlayerData>::from(camera_data.clone());
        buffer.write_buffer(&render_device, &render_queue);

        commands.insert_resource(PlayerDataUniform(buffer));
    }
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let voxel_data_bind_group_layout = world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let physics_data_bind_group_layout = world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: ShaderStages::COMPUTE,
                    //     ty: BindingType::Buffer {
                    //         ty: BufferBindingType::Uniform,
                    //         has_dynamic_offset: false,
                    //         min_binding_size: None,
                    //     },
                    //     count: None,
                    // },
                ],
            });
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        {

        }
        let raycast_shader = world
            .resource::<AssetServer>()
            .load("shaders/raytrace.wgsl");
        let physics_shader = world
            .resource::<AssetServer>()
            .load("shaders/physics.wgsl");
        let buffer_swap_shader = world
            .resource::<AssetServer>()
            .load("shaders/buffer_swap.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let compute_physics = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                voxel_data_bind_group_layout.clone(),
                physics_data_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: physics_shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });
        let compute_buffer_swap = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                voxel_data_bind_group_layout.clone(),
                physics_data_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: buffer_swap_shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });
        let compute_raycast = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                voxel_data_bind_group_layout.clone(),
                texture_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: raycast_shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        ComputePipeline {
            voxel_data_bind_group_layout,
            physics_data_bind_group_layout,
            texture_bind_group_layout,
            compute_raycast,
            compute_physics,
            compute_buffer_swap,
        }
    }
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    voxel_grid: Res<VoxelGridStorage>,
    double_buffer: Res<VoxelGridStorageDouble>,
    camera_data: Res<PlayerDataUniform>,
    raycast_image: Res<RaycastOutputImage>,
    render_device: Res<RenderDevice>,
) {
    // Bind the voxel data as a storage buffer
    {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.voxel_data_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &voxel_grid.0.buffer().unwrap(),
                    offset: 0,
                    size: None,
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &camera_data.0.buffer().unwrap(),
                    offset: 0,
                    // size: NonZeroU64::new(VOXEL_BUFFER_SIZE as u64),
                    size: None,
                }),
            },

            ],
        });
        commands.insert_resource(VoxelGridStorageBindGroup(bind_group));
    }
    // Bind the double buffer of the voxel grid
    {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.physics_data_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &double_buffer.0.buffer().unwrap(),
                    offset: 0,
                    size: None,
                }),
            },

            ],
        });
        commands.insert_resource(VoxelGridStorageDoubleBindGroup(bind_group));
    }

    // Bind the raycast result image as a texture
    {
        let view = &gpu_images[&raycast_image.0];
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view.texture_view),
            }],
        });
        commands.insert_resource(RaycastImageBindGroup(bind_group));
    }

}

#[derive(Default)]
struct RayCastRenderNode;

impl render_graph::Node for RayCastRenderNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let voxel_data_bind_group = &world.resource::<VoxelGridStorageBindGroup>().0;
        let physics_data_bind_group = &world.resource::<VoxelGridStorageDoubleBindGroup>().0;
        let texture_bind_group = &world.resource::<RaycastImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let physics_timer = world.resource::<PhysicsTimer>();

        // physics pass
        if physics_timer.triggered() {
            // First pass
            {
                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, voxel_data_bind_group, &[]);
                pass.set_bind_group(1, physics_data_bind_group, &[]);

                let compute_physics = pipeline_cache
                    .get_compute_pipeline(pipeline.compute_physics)
                    .unwrap();
                pass.set_pipeline(compute_physics);
                pass.dispatch_workgroups(VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE);
                // pass.dispatch_workgroups(1, 1, 1);
            }
            // Swap pass
            {
                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, voxel_data_bind_group, &[]);
                pass.set_bind_group(1, physics_data_bind_group, &[]);

                let compute_buffer_swap = pipeline_cache
                    .get_compute_pipeline(pipeline.compute_buffer_swap)
                    .unwrap();
                pass.set_pipeline(compute_buffer_swap);
                pass.dispatch_workgroups(VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE);
            }

        }

        // raycast pass
        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, voxel_data_bind_group, &[]);
            pass.set_bind_group(1, texture_bind_group, &[]);

            let compute_raycast = pipeline_cache
                .get_compute_pipeline(pipeline.compute_raycast)
                .unwrap();
            pass.set_pipeline(compute_raycast);
            pass.dispatch_workgroups(SCREEN_SIZE.0 / WORKGROUP_SIZE, SCREEN_SIZE.1 / WORKGROUP_SIZE, 1);
        }

        Ok(())
    }
}
