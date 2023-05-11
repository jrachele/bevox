#import "shaders/sand.wgsl"

@group(1) @binding(0)
var<uniform> physics: PhysicsData;

struct PhysicsData {
    time: f32,
    delta_seconds: f32,
}


fn handle_voxel_physics(index: vec3<i32>, voxel: u32) {
    // TODO: Explore if this is really the best way to do this.
    // Ensure we are performing physics only a certain number of times per second
    var t = 0.0;
    let frames = 60.0;
    let target_time = 1.0 / frames;

    // Multiply by two and check if we are on the other side of it, to avoid floating point precision mistakes
    let progress = physics.time % (2.0 * target_time);
    if (progress >= target_time) {
        return;
    }

    switch voxel {
        case 0u, default {
            return;
        }
        case 1u {
            handle_sand(index);
        }
    }
}
