#import "shaders/voxel.wgsl"
#import "shaders/player.wgsl"

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;

fn ray_grid_intersection(ray_origin: vec3<f32>, ray_direction: vec3<f32>, grid_position: vec3<f32>, grid_size: vec3<f32>) -> vec3<f32> {
    let t_min: vec3<f32> = (grid_position - ray_origin) / ray_direction;
    let t_max: vec3<f32> = (grid_position + grid_size - ray_origin) / ray_direction;

    let t_enter: f32 = max(max(min(t_min.x, t_max.x), min(t_min.y, t_max.y)), min(t_min.z, t_max.z));
    let t_exit: f32 = min(min(max(t_min.x, t_max.x), max(t_min.y, t_max.y)), max(t_min.z, t_max.z));

    if (t_enter > t_exit || t_exit < 0.0) {
        // No intersection with the grid
        return vec3<f32>(-100.0);
    }

    return ray_origin + ray_direction * t_enter;
}

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

    let grid_size = vec3<f32>(dim);
    let boundary_bottom_left = vec3<i32>(grid_pos);
    let boundary_top_right = vec3<i32>(grid_pos + (grid_size * VOXEL_SIZE));

    var voxel_position = vec3<i32>(ray_start.xyz);
    if (voxel_position.x < boundary_bottom_left.x || voxel_position.y < boundary_bottom_left.y || voxel_position.z < boundary_bottom_left.z ||
        voxel_position.x >= boundary_top_right.x || voxel_position.y >= boundary_top_right.y || voxel_position.z >= boundary_top_right.z) {
        voxel_position = vec3<i32>(ray_grid_intersection(ray_start.xyz, ray_direction, grid_pos, grid_size));
    }


    let delta_dist = abs(1.0 / (ray_direction * VOXEL_SIZE));
    let step = vec3<i32>(sign(ray_direction) * VOXEL_SIZE);
    var side_dist = (sign(ray_direction) * (vec3<f32>(voxel_position) - ray_start.xyz) + (sign(ray_direction) * 0.5) + 0.5) * delta_dist;

    var color = vec4<f32>(0.0);
    let maxSteps = u32(dim * 2.0);
    var mask = vec3<bool>(false);
    let center_pixel = ndc_space.x == 0.0 && ndc_space.y == 0.0;
    for (var i = 0u; i < maxSteps; i++) {
        if (voxel_position.x < boundary_bottom_left.x || voxel_position.x > boundary_top_right.x ||
            voxel_position.y < boundary_bottom_left.y || voxel_position.y > boundary_top_right.y ||
            voxel_position.z < boundary_bottom_left.z || voxel_position.z > boundary_top_right.z) {
            break;
        }

        // let index = vec3<f32>(vec3<f32>((voxel_position-vec3<i32>(grid_pos))) / VOXEL_SIZE);
        let index = vec3<f32>(voxel_position);
        var voxel = voxel_grid.voxels[get_index(vec3<i32>(index))];
        if (out_of_bounds(vec3<i32>(index))) {
            voxel = EMPTY_VOXEL;
            // break;
        }
        if (voxel != EMPTY_VOXEL) {
            color = vec4<f32>(get_voxel_color(voxel), 1.0);
            // if (get_voxel_type(voxel) == 1u) {
            //     color.w = 0.1;
            // }

            let center_voxel_already_selected = voxel_grid.selected.x == index.x && voxel_grid.selected.y == index.y && voxel_grid.selected.z == index.z;
            if (center_pixel) {
                voxel_grid.selected = index;
            }

            // TODO: Render brush as sphere with radius, in separate function
            if (center_voxel_already_selected) {
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

    }
    if (mask.y) {
        color *= 0.9;
    }
    if (mask.z) {
        color *= 0.75;
    }

    // if (color.x != 0.0 || color.y != 0.0 || color.z != 0.0) {
    //     color.w = 1.0;
    // }

    if (center_pixel) {
        voxel_grid.normal = vec3<f32>(0.0);
        if (mask.x) {
            voxel_grid.normal.x = -sign(ray_direction.x);
        }
        else if (mask.y) {
            voxel_grid.normal.y = -sign(ray_direction.y);
        }
        else if (mask.z) {
            voxel_grid.normal.z = -sign(ray_direction.z);
        }
    }

    textureStore(output_texture, invocation_id.xy, color);
}

// TODO:
// Begrudgingly COPY and PASTE the above code and render the brush preview with raycasting
// because for whatever reason, you can't just have a reference to a storage buffer. Why????
