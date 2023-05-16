#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer VoxelInBuffer { uint voxels_in[]; };
layout(set = 0, binding = 1, rgba8) uniform writeonly image2D img;
// layout(set = 0, binding = 1) buffer VoxelOutBuffer { uint voxels_out[]; };

layout(push_constant) uniform PushConstants {
    uint dim;
    mat4 camera_matrix;
    mat4 inverse_projection_matrix;
    uint mouse_click;
    uint brush_size;
} push_constants;

#include "voxel.glsl"

vec3 rayGridIntersection(vec3 rayOrigin, vec3 rayDirection, vec3 gridSize) {
    vec3 tMin = (vec3(0.0) - rayOrigin) / rayDirection;
    vec3 tMax = (gridSize - rayOrigin) / rayDirection;

    float tEnter = max(max(min(tMin.x, tMax.x), min(tMin.y, tMax.y)), min(tMin.z, tMax.z));
    float tExit = min(min(max(tMin.x, tMax.x), max(tMin.y, tMax.y)), max(tMin.z, tMax.z));

    if (tEnter > tExit || tExit < 0.0) {
        // No intersection with the grid
        return vec3(-1.0);
    }

    return rayOrigin + rayDirection * tEnter;
}

void main() {
    float dim = float(push_constants.dim);
    // vec3 grid_pos = voxel_grid.pos;
    vec3 grid_pos = vec3(0.0);
    vec2 pixel_coords = vec2(gl_GlobalInvocationID.xy);

    mat4 camera_matrix = push_constants.camera_matrix;
    mat4 inverse_projection_matrix = push_constants.inverse_projection_matrix;

    vec2 screen_size = vec2(imageSize(img));
    vec2 ndc_space = ((vec2(pixel_coords.x, pixel_coords.y) / screen_size) * 2.0) - vec2(1.0);

    vec4 ray_start = camera_matrix * inverse_projection_matrix * vec4(ndc_space, 0.0, 1.0);
    vec4 ray_end = camera_matrix * inverse_projection_matrix * vec4(ndc_space, 1.0, 1.0);
    vec3 ray_direction = normalize((ray_end.xyz / ray_end.w) - (ray_start.xyz / ray_start.w));

    vec3 grid_size = vec3(dim);

    // Ensure we start at the valid intersection point
    ivec3 voxel_position = ivec3(ray_start.xyz);
    if (voxel_position.x < 0 || voxel_position.x >= int(grid_size.x) ||
        voxel_position.y < 0 || voxel_position.y >= int(grid_size.y) ||
        voxel_position.z < 0 || voxel_position.z >= int(grid_size.z)) {
        voxel_position = ivec3(rayGridIntersection(ray_start.xyz, ray_direction, grid_size));
    }

    vec3 delta_dist = abs(1.0 / ray_direction);
    ivec3 ray_step = ivec3(sign(ray_direction));
    vec3 side_dist = (sign(ray_direction) * (voxel_position - ray_start.xyz) + (sign(ray_direction) * 0.5) + 0.5) * delta_dist;

    vec4 color = vec4(0.0);
    uint maxSteps = uint(dim * 2.0);
    bvec3 mask = bvec3(false);
    // bool center_pixel = ndc_space.x == 0.0 && ndc_space.y == 0.0;

    for (uint i = 0u; i < maxSteps; i++) {
        uint voxel = voxels_in[get_index(voxel_position)];

        if (voxel != EMPTY_VOXEL) {
            color = vec4(get_voxel_color(voxel), 1.0);
            break;
        }

        if (side_dist.x < side_dist.y) {
            if (side_dist.x < side_dist.z) {
                side_dist.x += delta_dist.x;
                voxel_position.x += ray_step.x;
                mask = bvec3(true, false, false);
            } else {
                side_dist.z += delta_dist.z;
                voxel_position.z += ray_step.z;
                mask = bvec3(false, false, true);
            }
        } else {
            if (side_dist.y < side_dist.z) {
                side_dist.y += delta_dist.y;
                voxel_position.y += ray_step.y;
                mask = bvec3(false, true, false);
            } else {
                side_dist.z += delta_dist.z;
                voxel_position.z += ray_step.z;
                mask = bvec3(false, false, true);
            }
        }

        if (voxel_position.x < 0 || voxel_position.x >= int(grid_size.x) ||
            voxel_position.y < 0 || voxel_position.y >= int(grid_size.y) ||
            voxel_position.z < 0 || voxel_position.z >= int(grid_size.z)) {
            break;
        }

    }

    if (mask.y == true) {
        color *= 0.9;
    }
    if (mask.z == true) {
        color *= 0.75;
    }

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), color);
}
