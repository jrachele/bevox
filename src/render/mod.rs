use std::borrow::Cow;
use std::sync::Arc;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{RenderGraph, self};
use bevy::render::render_resource::{StorageBuffer, ShaderType, UniformBuffer, BindGroup, BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BufferBindingType, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BindGroupEntry, BindGroupDescriptor, BufferBinding, BindingResource, ComputePassDescriptor, Extent3d, TextureDimension, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue, RenderContext};
use bevy::render::{RenderApp, RenderSet};
use bevy::render::extract_resource::{ExtractResourcePlugin, ExtractResource};

use bevy_app_compute::prelude::{ComputeShader, ShaderRef, ComputeWorker, AppComputeWorkerBuilder};
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

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct PhysicsComputeShader;

impl ComputeShader for PhysicsComputeShader {
    fn shader() -> ShaderRef {
        "shaders/physics.wgsl".into()
    }
}


#[derive(Resource)]
pub struct PhysicsComputeWorker;

impl ComputeWorker for PhysicsComputeWorker {
    fn build(world: &mut World) -> bevy_app_compute::prelude::AppComputeWorker<Self> {
        // let player_data = world.get_resource::<PlayerData>().expect("Unable to get player data!");
        // Create the 2D texture buffer to render the results of the raycast

        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
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
        let image_handle = images.add(image);

        world.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32)),
                ..default()
            },
            texture: image_handle.clone(),
            ..default()
        });

        let player_data = PlayerData::default();
        let voxel_data = precalculate_voxel_data();
        let workgroups = [VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE, VOXEL_GRID_SIZE / WORKGROUP_SIZE];
        let worker = AppComputeWorkerBuilder::new(world)
            .add_uniform("player_data", &player_data)
            .add_rw_storage("voxel_grid", &voxel_data)
            .add_rw_storage("out_grid", &voxel_data)
            .add_image("out_image", &image_handle)
            .add_pass::<PhysicsComputeShader>(workgroups, &["voxel_grid", "out_grid", "player_data"])
            .add_swap("voxel_grid", "out_grid")
            .build();

        worker
    }
}

fn precalculate_voxel_data() -> VoxelGrid {
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
    voxels
}

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

const SCREEN_SIZE: (u32, u32) = (1920, 1080);
const VOXEL_GRID_SIZE: u32 = 128u32;
const WORKGROUP_SIZE: u32 = 8;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {


    // Set up a timer to compute physics at a fixed interval
    let physics_timer = PhysicsTimer {
        elapsed_time: 0.0,
        trigger_time: 1.0 / 30.0
    };
    commands.insert_resource(physics_timer);

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

fn update_player_data(
    mut player_data: ResMut<PlayerData>,
    transform_query: Query<&Transform, With<FlyCam>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if let Ok(transform) = transform_query.get_single() {
        player_data.camera_matrix = transform.compute_matrix();
        let perspective_matrix = create_perspective_projection_matrix(16.0 / 9.0, 60.0, 0.1, 1000.0);
        player_data.inverse_perspective_matrix = perspective_matrix.inverse();
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
    player_data.mouse_click = mouse_buttons;

    player_data.brush_size = 3;

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

