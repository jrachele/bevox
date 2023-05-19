use bevy::{
    prelude::*,
    diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, Diagnostics}, window::PresentMode,
};
use bevy_app_compute::prelude::{AppComputePlugin, AppComputeWorkerPlugin};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, bevy_egui::EguiContexts, egui::{self, Ui}};
use util::flycam::{PlayerPlugin, MovementSettings, KeyBindings, FlyCam};
use render::{PhysicsComputeWorker};

// #[cfg(test)]
// mod tests;

mod util;
mod voxel;
mod render;

fn main() {
    App::new()
        // .insert_resource(ClearColor(Color::rgb(0.4, 0.75, 0.9)))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
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
            speed: 12.0, // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::LShift,
            ..Default::default()
        })
        .add_plugin(AppComputePlugin)
        // .add_plugin(AppComputeWorkerPlugin::<PhysicsComputeWorker>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(diagnostic_ui)
        .run();
}
/// Give our text a custom size
fn sized_text(ui: &mut Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size));
}

/// System to generate user interface with egui
pub fn diagnostic_ui(
    mut contexts: EguiContexts,
    diagnostics: Res<Diagnostics>,
    transform_query: Query<&Transform, With<FlyCam>>
) {
    let ctx = contexts.ctx_mut();
    egui::Area::new("fps")
        .fixed_pos(egui::pos2(10.0, 10.0))
        .show(&ctx, |ui| {
            let size = 15.0;
            if let Some(diag) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(avg) = diag.average() {
                    sized_text(ui, format!("FPS: {:.2}", avg), size);
                }
            }
            if let Ok(transform) = transform_query.get_single() {
                sized_text(ui,
                           format!("x: {}, y: {}, z: {}", transform.translation.x, transform.translation.y, transform.translation.z),
                           size);
            }
        });
}
