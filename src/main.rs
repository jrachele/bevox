use bevy::{prelude::*, window::{close_on_esc, WindowMode}};
use bevy_vulkano::{BevyVulkanoSettings, VulkanoWinitPlugin, BevyVulkanoWindows, BevyVulkanoContext};
use gui::user_interface;
use physics::PhysicsComputePipeline;
use render::{raytracing_pipeline::RayTracingComputePipeline, place_over_frame::RenderPassPlaceOverFrame};
use util::{flycam::{PlayerPlugin, MovementSettings, KeyBindings, FlyCam}, create_perspective_projection_matrix};

mod gui;
mod render;
mod util;
mod physics;
mod voxel;

pub const WIDTH: f32 = 1920.0;
pub const HEIGHT: f32 = 1080.0;
pub const VOXEL_GRID_DIM: u32 = 128;

fn main() {
    App::new()
        // Resources
        .insert_non_send_resource(BevyVulkanoSettings {
            is_gui_overlay: true,
            ..default()
        })
        .insert_resource(PhysicsTimer::new(1.0 / 60.0))
        // Add needed (minimum) plugins
        .add_plugin(bevy::core::TaskPoolPlugin::default())
        .add_plugin(bevy::core::TypeRegistrationPlugin::default())
        .add_plugin(bevy::core::FrameCountPlugin::default())
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::log::LogPlugin::default())
        .add_plugin(bevy::time::TimePlugin)
        .add_plugin(bevy::input::InputPlugin)
        // Bevy-managed window
        .add_plugin(WindowPlugin { primary_window: Some(Window {
            resolution: (WIDTH, HEIGHT).into(),
            title: "Voxel Engine".to_string(),
            present_mode: bevy::window::PresentMode::Immediate,
            resizable: false,
            mode: WindowMode::Windowed,
            ..default()
        }), ..default() })
        // The plugin to use vulkano with bevy
        .add_plugin(VulkanoWinitPlugin)
        // Add GUI
        .add_system(user_interface)
        // Misc
        .add_system(update_physics_timer)
        // Camera
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
        .init_resource::<PlayerData>()
        .add_startup_system(precalculate_inverse_projection)
        .add_system(update_player_data)
        // Compute stuff
        .add_startup_system(create_pipelines)
        .add_system(render.in_base_set(CoreSet::PostUpdate))
        .run();
}

/// Creates our simulation pipeline & render pipeline
fn create_pipelines(
    mut commands: Commands,
    window_query: Query<Entity, With<Window>>,
    context: Res<BevyVulkanoContext>,
    windows: NonSend<BevyVulkanoWindows>,
) {
    let window_entity = window_query.single();
    let primary_window = windows.get_vulkano_window(window_entity).unwrap();
    // Create compute pipeline to simulate game of life
    let physics_pipeline = PhysicsComputePipeline::new(
        context.context.memory_allocator(),
        primary_window.renderer.graphics_queue(),
    );

    let voxel_grid_buffer = physics_pipeline.get_voxel_grid();

    // Create our compute pipeline for raycasting
    let render_pipeline = RayTracingComputePipeline::new(
        context.context.memory_allocator(),
        primary_window.renderer.graphics_queue(),
        voxel_grid_buffer.clone()
    );

    // Create our render pass
    let place_over_frame = RenderPassPlaceOverFrame::new(
        context.context.memory_allocator().clone(),
        primary_window.renderer.graphics_queue(),
        primary_window.renderer.swapchain_format(),
    );

    // Insert resources
    commands.insert_resource(physics_pipeline);
    commands.insert_resource(render_pipeline);
    commands.insert_resource(place_over_frame);
}

#[derive(Resource)]
struct PhysicsTimer {
    elapsed_time: f32,
    trigger_time: f32
}

impl PhysicsTimer {
    pub fn new(trigger_time: f32) -> Self {
        Self {
            elapsed_time: 0.0,
            trigger_time
        }
    }
    pub fn triggered(&self) -> bool {
        self.elapsed_time >= self.trigger_time
    }

    pub fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }

    pub fn tick(&mut self, amt: f32) {
        self.elapsed_time += amt;
    }
}

fn update_physics_timer(mut timer: ResMut<PhysicsTimer>, time: Res<Time>) {
    if timer.triggered() {
        timer.reset();
    }

    timer.tick(time.delta_seconds());
}

#[derive(Default, Clone, Resource)]
struct PlayerData {
    pub camera_matrix: Mat4,
    pub inverse_projection_matrix: Mat4,
    pub mouse_click: u32,
    pub brush_size: u32,
}

#[derive(Component)]
struct InvProjectionMatrix(Mat4);

fn precalculate_inverse_projection(mut commands: Commands) {
    commands.spawn(
        InvProjectionMatrix(
            create_perspective_projection_matrix(16.0 / 9.0, 90.0, 0.1, 1000.0)
                .inverse()));
}

fn update_player_data(mut player_data: ResMut<PlayerData>,
                      transform_query: Query<&Transform, With<FlyCam>>,
                      matrix_query: Query<&InvProjectionMatrix>,
                      mouse_input: Res<Input<MouseButton>>) {
    let camera_transform = transform_query.single();
    let inv_proj_matrix = matrix_query.single();

    player_data.camera_matrix = camera_transform.compute_matrix();
    player_data.inverse_projection_matrix = inv_proj_matrix.0;

    let mut mouse_buttons = 0;
    if mouse_input.pressed(MouseButton::Left) {
        mouse_buttons |= 1 << 1;
    }
    if mouse_input.pressed(MouseButton::Middle) {
        mouse_buttons |= 1 << 2;
    }
    if mouse_input.pressed(MouseButton::Right) {
        mouse_buttons |= 1 << 3;
    }

    player_data.mouse_click = mouse_buttons;

    // TODO: Make this adjustable
    player_data.brush_size = 3;
}

/// Render the simulation
fn render(window_query: Query<Entity, With<Window>>,
          physics_timer: Res<PhysicsTimer>,
          player_data: Res<PlayerData>,
          mut physics_pipeline: ResMut<PhysicsComputePipeline>,
          mut render_pipeline: ResMut<RayTracingComputePipeline>,
          mut place_over_frame: ResMut<RenderPassPlaceOverFrame>,
          mut vulkano_windows: NonSendMut<BevyVulkanoWindows>) {
    let window_entity = window_query.single();
    let primary_window = vulkano_windows.get_vulkano_window_mut(window_entity).unwrap();

    // Start frame
    let before = match primary_window.renderer.acquire() {
        Err(e) => {
            bevy::log::error!("Failed to start frame: {}", e);
            return;
        }
        Ok(f) => f,
    };

    // Only dispatch the physics compute shader when the timer is triggered
    // to limit its frames
    let after_physics = match physics_timer.triggered() {
        true => physics_pipeline.compute(before),
        false => before
    };
    // let after_physics = before;


    let after_raycast = render_pipeline.compute(after_physics, player_data.clone());

    let color_image = render_pipeline.get_result_image();

    // Access the final window image (this is the current GPU image which changes between frames (swapchain))
    let final_image = primary_window.renderer.swapchain_image_view();

    // Draw the raycast result to the screen
    let after_render = place_over_frame.render(after_raycast, color_image, final_image.clone());

    // Draw GUI using egui_winit_window's GUI draw pipeline
    let after_gui = primary_window.gui.draw_on_image(after_render, final_image);

    // Finish Frame
    primary_window.renderer.present(after_gui, true);
}
