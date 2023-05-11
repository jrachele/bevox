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
    }, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, window::PresentMode,
};
use flycam::{PlayerPlugin, MovementSettings, KeyBindings, FlyCam};
use render::RenderComputePlugin;
use voxel::VoxelGrid;

#[cfg(test)]
mod tests;

mod flycam;
mod voxel;
mod render;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Engine".to_string(),
                resolution: (1920., 1080.).into(),
                resizable: false,
                present_mode: PresentMode::AutoNoVsync,
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
        .add_plugin(RenderComputePlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

