//! A shader that renders a mesh multiple times in one draw call.

use std::borrow::Cow;

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
            TextureUsages, TextureViewDimension,
        },
        renderer::{RenderContext, RenderDevice},
        RenderApp, RenderSet,
    },
};
use voxel::VoxelGrid;

#[cfg(test)]
mod tests;

mod voxel;

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;
const VOXEL_BUFFER_SIZE: usize = std::mem::size_of::<u32>() * 64 * 64 * 64;

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
        }))
        .add_plugin(RayCastComputePlugin)
        .add_startup_system(setup)
        // .add_system(grid_test)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    let n = 10;
    let mut voxels = VoxelGrid::new(n);

    // Create a diagonal line for testing purposes
    for i in 0..n {
        if let Some(mut voxel) = voxels.get_mut(i, i, i) {
            voxel.0 = 1;
        }
    }

    // Create a storage buffer containing our voxel data
    let pos = Vec3 {
        x: 150.0,
        y: 200.0,
        z: 0.0,
    };
    let binding = [pos];
    let pos_bytes = bytemuck::cast_slice(&binding);
    let voxel_bytes = bytemuck::cast_slice(voxels.voxels.as_slice());

    // Pad with extra 4 bytes with &[0u8] as WGSL structs are aligned by powers of 2
    let buffer_contents = &[pos_bytes, &[0u8], voxel_bytes].concat();

    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("voxel data buffer"),
        contents: buffer_contents,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    commands.insert_resource(VoxelGridStorage(buffer));

    // Create the 2D texture buffer to render the results of the raycast
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
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
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
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
        app.add_plugin(ExtractResourcePlugin::<RaycastImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RaycastPipeline>()
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raycast", RayCastRenderNode::default());
        render_graph.add_node_edge("raycast", bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct VoxelGridStorage(Buffer);

#[derive(Resource, Clone, ExtractResource)]
struct VoxelUniforms {
    pos: Vec3,
    buffer: Buffer,
}

// fn grid_test(mut grid: ResMut<VoxelGridStorage>, time: Res<Time>) {
//     grid.pos.x += time.elapsed_seconds() % SIZE.0 as f32;
// }

#[derive(Resource)]
struct VoxelGridDataBindGroup(BindGroup);

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

impl FromWorld for RaycastPipeline {
    fn from_world(world: &mut World) -> Self {
        println!("from_world called!");
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
    voxel_data: Res<VoxelGridStorage>,
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
                    buffer: &voxel_data.0,
                    offset: 0,
                    // size: NonZeroU64::new(VOXEL_BUFFER_SIZE as u64),
                    size: None,
                }),
            },
            ],
        });
        commands.insert_resource(VoxelGridDataBindGroup(bind_group));
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
        let voxel_data_bind_group = &world.resource::<VoxelGridDataBindGroup>().0;
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
        pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

        Ok(())
    }
}
