use bevy::{
    prelude::*,
    diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, window::PresentMode,
};
use util::flycam::{PlayerPlugin, MovementSettings, KeyBindings, FlyCam};
use render::RenderComputePlugin;

// #[cfg(test)]
// mod tests;

mod util;
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

