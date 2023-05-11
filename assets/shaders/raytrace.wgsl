#import "shaders/voxel.wgsl"
#import "shaders/player.wgsl"

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let dim = f32(voxel_grid.dim);
    let grid_pos = voxel_grid.pos;
    let pixel_coords = invocation_id.xy;

    let camera_matrix = player_data.camera_matrix;
    let inverse_projection_matrix = player_data.inverse_projection_matrix;

    let screen_size = vec2<f32>(textureDimensions(output_texture));
    let ndc_space = ((vec2<f32>(f32(pixel_coords.x), screen_size.y - f32(pixel_coords.y)) / screen_size) * 2.0) - vec2<f32>(1.0);

    let ray_start = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 0.0, 1.0);
    let ray_end = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 1.0, 1.0);
    let ray_direction = normalize((ray_end.xyz / ray_end.w) - (ray_start.xyz / ray_start.w));

    // let boundary_top_right = voxel_grid.pos + vec3<f32>(dim);
    let boundary_top_right = (grid_pos + vec3<f32>(VOXEL_SIZE * dim));
    let boundary_bottom_left = (grid_pos);

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

    let delta_dist = abs(1.0 / (ray_direction * VOXEL_SIZE));
    // let step = sign(ray_direction);
    let step = sign(ray_direction) * VOXEL_SIZE;
    var side_dist = (sign(ray_direction) * (voxel_position - ray_start.xyz) + (sign(ray_direction) * 0.5) + 0.5) * delta_dist;
    // var side_dist = (sign(ray_direction) * (voxel_position - ray_start.xyz) + (sign(ray_direction) * 0.5 * VOXEL_SIZE) + 0.5 * VOXEL_SIZE) * delta_dist;
    // var side_dist = (step * (voxel_position - ray_start.xyz) + (step * 0.5) + 0.5) * delta_dist;

    var color = vec4<f32>(0.0);
    let maxSteps = u32(dim * 2.0);
    var mask = vec3<bool>(false);
    let center_pixel = ndc_space.x == 0.0 && ndc_space.y == 0.0;
    for (var i = 0u; i < maxSteps; i++) {
        let index = vec3<f32>((voxel_position-grid_pos) / VOXEL_SIZE);
        let flat_index = (index.x * dim * dim) + (index.y * dim) + index.z;
        let voxel = voxel_grid.voxels[u32(flat_index)];
        if (voxel > 0u) {
            color = vec4<f32>(0.5, 0.3, 0.1, 1.0);

            // TODO: Store index in read_write buffer for brush
            if (center_pixel) {
                voxel_grid.selected = index;
            }

            if (voxel_grid.selected.x == index.x && voxel_grid.selected.y == index.y && voxel_grid.selected.z == index.z) {
                color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
            }
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

    // TODO: Maybe calculate normals some other way later?
    if (center_pixel) {
        if (mask.x) {
            voxel_grid.normal.x = 1.0;
        }
        else if (mask.y) {
            voxel_grid.normal.y = 1.0;
        }
        else if (mask.z) {
            voxel_grid.normal.z = 1.0;
        }
    }

    textureStore(output_texture, invocation_id.xy, color);
}

// TODO:
// Begrudgingly COPY and PASTE the above code and render the brush preview with raycasting
// because for whatever reason, you can't just have a reference to a storage buffer. Why????
