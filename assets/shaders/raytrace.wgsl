@group(0) @binding(0)
var<storage, read> voxel_grid: VoxelGrid;

@group(0) @binding(1)
var<uniform> camera_data: CameraData;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;

const VOXEL_SIZE: f32 = 1.0;

struct VoxelGrid {
    dim: u32,
    pos: vec3<f32>,
    voxels: array<u32>
}

struct CameraData {
    camera_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
}


@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    // Invert the camera matrix to obtain the view matrix. The view matrix represents the camera_matrixation from world-space to camera-space.
    // Compute the inverse of the projection matrix. The projection matrix maps points in camera-space to NDC (normalized device coordinates) space.

    // Convert the screen-space coordinates (x, y) to NDC-space by dividing by the viewport dimensions and mapping the resulting values to the range [-1, 1] along the x and y axes.

    // Construct a 4D vector (x_ndc, y_ndc, z_ndc, 1) with z_ndc = 0 (for a point on the near clipping plane).

    // Multiply the vector by the inverse projection matrix to obtain a 4D vector (x_camera, y_camera, z_camera, w_camera) in camera-space.

    // Divide the resulting vector by its fourth component w_camera to obtain a 3D point (x_camera / w_camera, y_camera / w_camera, z_camera / w_camera) in camera-space.

    // Multiply the resulting point by the view matrix to obtain a 3D point (x_world, y_world, z_world) in world-space.

    let camera_matrix = camera_data.camera_matrix;
    let view_matrix = camera_data.view_matrix;
    let projection_matrix = camera_data.projection_matrix;
    let inverse_projection_matrix = camera_data.inverse_projection_matrix;

    let screen_size = vec2<f32>(textureDimensions(output_texture));
    let ndc_space = ((vec2<f32>(f32(invocation_id.x), screen_size.y - f32(invocation_id.y)) / screen_size) * 2.0) - vec2<f32>(1.0);
    // let ndc_space = ((vec2<f32>(invocation_id.xy) / screen_size) * 2.0) - vec2<f32>(1.0);

    let ray_start = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 0.0, 1.0);
    let ray_end = camera_matrix * inverse_projection_matrix * vec4<f32>(ndc_space, 1.0, 1.0);
    // let ray_direction = normalize((ray_start.xyz / ray_start.w));
    let ray_direction = normalize((ray_end.xyz / ray_end.w) - (ray_start.xyz / ray_start.w));


    var voxel_position = floor(ray_start.xyz);

    let delta_dist = abs(vec3<f32>(length(ray_direction)) / ray_direction);
    // let max_component = max(max(abs(ray_direction.x), abs(ray_direction.y)), abs(ray_direction.z));
    // let step = sign(ray_direction) * (1.0 / vec3<f32>(max(max_component, 0.001)));
    let step = sign(ray_direction);
    var side_dist = (sign(ray_direction) * (voxel_position - ray_start.xyz) + (sign(ray_direction) * 0.5) + 0.5) * delta_dist;

    var color = vec4<f32>(0.0);
    let maxSteps = 512;
    var mask = vec3<bool>(false);
    for (var i = 0; i < maxSteps; i++) {
        // Get voxel data
        let dim = f32(voxel_grid.dim);
        let index = vec3<f32>((voxel_position-voxel_grid.pos) / VOXEL_SIZE);
        // let index = vec3<u32>(voxel_position);
        // let index = vec3<u32>((voxel_grid.pos - voxel_position) / VOXEL_SIZE);

        let flat_index = (index.x * dim * dim) + (index.y * dim) + index.z;
        if (flat_index >= 0.0 && flat_index < dim*dim*dim) {
            // let flat_index = index.x + index.y * dim + index.z * dim * dim;
            let voxel = voxel_grid.voxels[u32(flat_index)];
            if (voxel > 0u) {
                color = vec4<f32>(0.5, 0.5, 0.8, 1.0);
                continue;
            }
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

        // if (voxel_position )
    }
    if (mask.y) {
        color *= 0.5;
    }
    if (mask.z) {
        color *= 0.75;
    }

    textureStore(output_texture, invocation_id.xy, color);

}
