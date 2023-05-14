fn handle_water(index: vec3<i32>) {
    var index = index;
    var current_voxel = EMPTY_VOXEL;
    if (!get_voxel(index, &current_voxel) || current_voxel == EMPTY_VOXEL) {
        return;
    }
    let below_block_index = vec3<i32>(index.x, index.y - 1, index.z);

    let dim = i32(voxel_grid.dim);

    // First check if we can go straight down
    var below_block = EMPTY_VOXEL;
    if (get_voxel(below_block_index, &below_block) && below_block == EMPTY_VOXEL) {
        set_voxel(index, EMPTY_VOXEL);
        set_voxel(below_block_index, current_voxel);
        storageBarrier();
    }
    // If not check if we can go anywhere in the 3x3 blocks surrounding the current block
    else {
        for (var i = -1; i <= 1; i++) {
            let left_index = index.x + i;
            for (var k = -1; k <= 1; k++) {
                let right_index = index.z + k;
                let side_block_index = vec3<i32>(left_index, index.y - 1, right_index);
                var side_block = EMPTY_VOXEL;
                if (get_voxel(side_block_index, &side_block) && side_block == EMPTY_VOXEL) {
                    set_voxel(index, EMPTY_VOXEL);
                    set_voxel(side_block_index, current_voxel);
                    // index = side_block_index;
                    storageBarrier();
                    return;
                }
            }
        }
    }
}
