@group(0) @binding(0)
var<storage, read> voxel_grid: VoxelGrid;

@group(0) @binding(1)
var<uniform> camera_data: CameraData;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;

struct VoxelGrid {
    dim: u32,
    pos: vec3<f32>,
    voxels: array<u32>
}

struct CameraData {
    camera_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
}


@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let camera_matrix = camera_data.camera_matrix;
    let inverse_projection_matrix = camera_data.inverse_projection_matrix;

    let screen_size = vec2<f32>(textureDimensions(output_texture));
    let ndc_space = ((vec2<f32>(f32(invocation_id.x), screen_size.y - f32(invocation_id.y)) / screen_size) * 2.0) - vec2<f32>(1.0);

    let ray_start = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 0.0, 1.0);
    let ray_end = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 1.0, 1.0);
    let ray_direction = normalize((ray_end.xyz / ray_end.w) - (ray_start.xyz / ray_start.w));

    let dim = f32(voxel_grid.dim);
    let boundary_top_right = voxel_grid.pos + vec3<f32>(dim);
    let boundary_bottom_left = voxel_grid.pos;

    var voxel_position = floor(ray_start.xyz);

    // if the voxel position isn't in the grid, move it to the point of intersection
    if (voxel_position.x < boundary_bottom_left.x) {
        voxel_position.x = boundary_bottom_left.x;
    }
    if (voxel_position.x >= boundary_top_right.x) {
        voxel_position.x = boundary_top_right.x;
    }
    if (voxel_position.y < boundary_bottom_left.y) {
        voxel_position.y = boundary_bottom_left.y;
    }
    if (voxel_position.y >= boundary_top_right.y) {
        voxel_position.y = boundary_top_right.y;
    }
    if (voxel_position.z < boundary_bottom_left.z) {
        voxel_position.z = boundary_bottom_left.z;
    }
    if (voxel_position.z >= boundary_top_right.z) {
        voxel_position.z = boundary_top_right.z;
    }

    let delta_dist = abs(1.0 / ray_direction);
    let step = sign(ray_direction);
    var side_dist = (step * (voxel_position - ray_start.xyz) + (step * 0.5) + 0.5) * delta_dist;

    var color = vec4<f32>(0.0);
    let maxSteps = u32(dim * 2.0);
    var mask = vec3<bool>(false);
    for (var i = 0u; i < maxSteps; i++) {
        let index = vec3<f32>(voxel_position-floor(voxel_grid.pos));
        let flat_index = (index.x * dim * dim) + (index.y * dim) + index.z;
        let voxel = voxel_grid.voxels[u32(flat_index)];
        if (voxel > 0u) {
            color = vec4<f32>(0.5, 0.5, 0.8, 1.0);
            break;
        }

        if (side_dist.x < side_dist.y) {
            if (side_dist.x < side_dist.z) {
                side_dist.x += delta_dist.x;
                voxel_position.x += step.x;
                mask = vec3<bool>(true, false, false);
            } else {
                side_dist.z += delta_dist.z;
                voxel_position.z += step.z;
                mask = vec3<bool>(false, false, true);
            }
        } else {
            if (side_dist.y < side_dist.z) {
                side_dist.y += delta_dist.y;
                voxel_position.y += step.y;
                mask = vec3<bool>(false, true, false);
            } else {
                side_dist.z += delta_dist.z;
                voxel_position.z += step.z;
                mask = vec3<bool>(false, false, true);
            }
        }

        if (voxel_position.x < boundary_bottom_left.x || voxel_position.x > boundary_top_right.x ||
            voxel_position.y < boundary_bottom_left.y || voxel_position.y > boundary_top_right.y ||
            voxel_position.z < boundary_bottom_left.z || voxel_position.z > boundary_top_right.z) {
            break;
        }


    }
    if (mask.y) {
        color *= 0.5;
    }
    if (mask.z) {
        color *= 0.75;
    }

    textureStore(output_texture, invocation_id.xy, color);

}
