//! A shader that renders a mesh multiple times in one draw call.

use std::{borrow::Cow, rc::Rc, sync::Arc};

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
            BufferBinding, BufferBindingType, BufferInitDescriptor, BufferUsages,
            CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, Extent3d,
            PipelineCache, ShaderStages, StorageTextureAccess, TextureDimension, TextureFormat,
            TextureUsages, TextureViewDimension, StorageBuffer, ShaderType, UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        RenderApp, RenderSet,
    }, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
};
use flycam::{PlayerPlugin, MovementSettings, KeyBindings, FlyCam};
use voxel::VoxelGrid;

#[cfg(test)]
mod tests;

mod flycam;
mod voxel;

const SCREEN_SIZE: (u32, u32) = (1920, 1080);
const WORKGROUP_SIZE: u32 = 8;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Engine".to_string(),
                resolution: (1920., 1080.).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 3.0, // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::LShift,
            ..Default::default()
        })
        .add_plugin(RayCastComputePlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(update_camera_gpu)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {

    // Create a storage buffer containing our voxel data
    let pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    let n = 128;
    let r = (n - 1) as f32;
    let mut voxels = VoxelGrid::new(n, pos);

    // Create a diagonal line for testing purposes
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let pos = (Vec3::new(i as f32, j as f32, k as f32) * 2.0) - Vec3::splat((n) as f32);
                let in_sphere = pos.length_squared() <= r * r;
                if let Some(mut voxel) = voxels.get_mut(i, j, k) {
                    if in_sphere {
                        voxel.value = 1;
                    }
                }
            }
        }
    }

    let mut buffer = StorageBuffer::<VoxelGrid>::from(voxels);
    buffer.write_buffer(&render_device, &render_queue);

    commands.insert_resource(VoxelGridStorage(Arc::new(buffer)));

    let uniform = CameraData {
        camera_matrix: Mat4::default(),
        view_matrix: Mat4::default(),
        perspective_matrix: Mat4::default(),
        inverse_perspective_matrix: Mat4::default(),
    };

    commands.insert_resource(uniform);

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

    commands.insert_resource(RaycastImage(image));

    commands.spawn(Camera2dBundle::default());
}

pub struct RayCastComputePlugin;

impl Plugin for RayCastComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<VoxelGridStorage>::default());
        app.add_plugin(ExtractResourcePlugin::<CameraData>::default());
        app.add_plugin(ExtractResourcePlugin::<RaycastImage>::default());
        // app.register_type::<VoxelGrid>();
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RaycastPipeline>()
            .add_system(prepare_uniform_data.in_set(RenderSet::Prepare))
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raycast", RayCastRenderNode::default());
        render_graph.add_node_edge("raycast", bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct VoxelGridStorage(Arc<StorageBuffer<VoxelGrid>>);

#[derive(Resource, Clone, ShaderType, ExtractResource)]
struct CameraData {
    camera_matrix: Mat4,
    view_matrix: Mat4,
    perspective_matrix: Mat4,
    inverse_perspective_matrix: Mat4,
}

#[derive(Resource)]
struct VoxelGridUniform(UniformBuffer<CameraData>);

#[derive(Resource)]
struct VoxelGridStorageBindGroup(BindGroup);

#[derive(Resource, Clone, Deref, ExtractResource)]
struct RaycastImage(Handle<Image>);

#[derive(Resource)]
struct RaycastImageBindGroup(BindGroup);

#[derive(Resource)]
pub struct RaycastPipeline {
    voxel_data_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

fn update_camera_gpu(
    mut uniform_data: ResMut<CameraData>,
    transform_query: Query<&Transform, With<FlyCam>>,
) {
    if let Ok(transform) = transform_query.get_single() {
        println!("{:?}", transform);

        uniform_data.camera_matrix = transform.compute_matrix();
        uniform_data.view_matrix = transform.compute_matrix().inverse();
        let perspective_matrix = create_perspective_projection_matrix(16.0 / 9.0, 60.0, 0.1, 1000.0);
        uniform_data.perspective_matrix = perspective_matrix.clone();
        uniform_data.inverse_perspective_matrix = perspective_matrix.inverse();
    }
}

fn prepare_uniform_data(
    mut commands: Commands,
    uniform_data: ResMut<CameraData>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let mut buffer = UniformBuffer::<CameraData>::from(uniform_data.clone());
    buffer.write_buffer(&render_device, &render_queue);

    commands.insert_resource(VoxelGridUniform(buffer));
}

impl FromWorld for RaycastPipeline {
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
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            // min_binding_size: NonZeroU64::new(VOXEL_BUFFER_SIZE as u64),
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
                            // min_binding_size: NonZeroU64::new(VOXEL_BUFFER_SIZE as u64),
                            min_binding_size: None,
                        },
                        count: None,
                    },
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
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/raytrace.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                voxel_data_bind_group_layout.clone(),
                texture_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        RaycastPipeline {
            voxel_data_bind_group_layout,
            texture_bind_group_layout,
            update_pipeline,
        }
    }
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<RaycastPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    voxel_grid: Res<VoxelGridStorage>,
    camera_data: Res<VoxelGridUniform>,
    raycast_image: Res<RaycastImage>,
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
                    // size: NonZeroU64::new(VOXEL_BUFFER_SIZE as u64),
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
        let texture_bind_group = &world.resource::<RaycastImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RaycastPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, voxel_data_bind_group, &[]);
        pass.set_bind_group(1, texture_bind_group, &[]);

        let update_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.update_pipeline)
            .unwrap();
        pass.set_pipeline(update_pipeline);
        // Dispatch n number of threads to take this shit down
        pass.dispatch_workgroups(SCREEN_SIZE.0 / WORKGROUP_SIZE, SCREEN_SIZE.1 / WORKGROUP_SIZE, 1);

        Ok(())
    }
}
