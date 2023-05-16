void handle_sand(ivec3 index) {
    uint current_voxel = voxels_in[get_index(index)];

    if (current_voxel == EMPTY_VOXEL) {
        return;
    }

    ivec3 below_block_index = ivec3(index.x, index.y - 1, index.z);

    int dim = int(push_constants.dim);

    // First check if we can go straight down
    if (!out_of_bounds(below_block_index)) {
        uint below_block = voxels_in[get_index(below_block_index)];
        if (below_block == EMPTY_VOXEL) {
            set_voxel(index, EMPTY_VOXEL);
            set_voxel(below_block_index, current_voxel);
            return;
        }
    }

    // // If not check if we can go anywhere in the 3x3 blocks surrounding the current block, underneath it
    for (int i = -1; i <= 1; i++) {
        int left_index = index.x + i;
        for (int k = -1; k <= 1; k++) {
            int right_index = index.z + k;
            int clumpiness = 2;
            ivec3 side_block_index = ivec3(left_index, index.y - clumpiness, right_index);
            // TODO: Add density check to determine if we can displace the neighboring voxel
            if (!out_of_bounds(side_block_index)) {
                uint side_block = voxels_in[get_index(side_block_index)];
                if (side_block == EMPTY_VOXEL || get_voxel_type(side_block) == VT_WATER) {
                    set_voxel(index, side_block);
                    set_voxel(side_block_index, current_voxel);
                    // storageBarrier();
                    return;
                }
            }
        }
    }

}
