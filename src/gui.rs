use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_vulkano::{
    egui_winit_vulkano::{egui, egui::Ui},
    BevyVulkanoWindows,
};

use crate::util::flycam::FlyCam;

/// Give our text a custom size
fn sized_text(ui: &mut Ui, text: impl Into<String>, size: f32) {
    ui.label(egui::RichText::new(text).size(size));
}

/// System to generate user interface with egui
pub fn user_interface(window_query: Query<Entity, With<Window>>,
    vulkano_windows: NonSend<BevyVulkanoWindows>,
    diagnostics: Res<Diagnostics>,
    transform_query: Query<&Transform, With<FlyCam>>
) {
    let window_entity = window_query.single();
    let primary_window = vulkano_windows.get_vulkano_window(window_entity).unwrap();
    let ctx = primary_window.gui.context();
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
