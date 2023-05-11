@group(0) @binding(1)
var<uniform> player_data: PlayerData;

struct PlayerData {
    camera_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
    mouse_click: u32,
    brush_size: u32,
}
