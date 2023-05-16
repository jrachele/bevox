#version 450

// layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;

layout(set = 0, binding = 0) buffer VoxelInBuffer { uint voxels_in[]; };
layout(set = 0, binding = 1) buffer VoxelOutBuffer { uint voxels_out[]; };

layout(push_constant) uniform PushConstants {
    uint dim;
} push_constants;

#include "voxel.glsl"

bool set_voxel(ivec3 index, uint voxel_data) {
    int dim = int(push_constants.dim);
    if (out_of_bounds(index)) {
        return false;
    }

    // Handle collisions: if there is already a block, don't move it
    // if (voxels_out[get_index(index)] != EMPTY_VOXEL) {
    //     return false;
    // }

    voxels_out[get_index(index)] = voxel_data;
    // memoryBarrierShared();
    return true;
}

#include "blocks/sand.glsl"
#include "blocks/water.glsl"

void handle_voxel_physics(ivec3 index, uint voxel_data) {
    if (voxel_data == EMPTY_VOXEL) {
        return;
    }

    switch (get_voxel_type(voxel_data)) {
        case VT_SAND:
            handle_sand(index);
            break;
        case VT_WATER:
            handle_water(index);
            break;
        default:
            break;
    }
}

const uint STAGE_COMPUTE = 0;
const uint STAGE_SWAP = 1;

void main() {
    ivec3 index = ivec3(gl_GlobalInvocationID);
    handle_voxel_physics(index, voxels_in[get_index(index)]);
    // int dim = int(push_constants.dim);
    // for (int i = 0; i < dim; i++) {
    //     for (int j = 0; j < dim; j++) {
    //         for (int k = 0; k < dim; k++) {
    //             ivec3 index = ivec3(i, j, k);
    //             handle_voxel_physics(index, voxels_in[get_index(index)]);
    //         }
    //     }
    // }
}
