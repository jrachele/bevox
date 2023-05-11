fn handle_sand(index: vec3<i32>) {
    let current_voxel = voxel_grid.voxels[get_index(index)];
    let below_block = vec3<i32>(index.x, index.y - 1, index.z);

    let dim = i32(voxel_grid.dim);
    if (index.y <= 1 || index.y >= dim) {
        return;
    }
    // First check if we can go straight down
    if (voxel_grid.voxels[get_index(below_block)] == 0u) {
        voxel_grid.voxels[get_index(index)] = 0u;
        voxel_grid.voxels[get_index(below_block)] = current_voxel;
    // If not check if we can go anywhere in the 3x3 blocks surrounding the current block, underneath it
    } else {
        for (var i = -1; i <= 1; i++) {
            let left_index = index.x + i;
            if (left_index <= 0 || left_index >= dim - 1) {
                continue;
            }
            for (var k = -1; k <= 1; k++) {
                let right_index = index.z + k;
                if (right_index <= 0 || right_index >= dim - 1) {
                    continue;
                }
                let block_index = vec3<i32>(left_index, index.y - 1, right_index);
                if (voxel_grid.voxels[get_index(block_index)] == 0u) {
                    voxel_grid.voxels[get_index(index)] = 0u;
                    voxel_grid.voxels[get_index(block_index)] = current_voxel;
                    return;
                }
            }
        }
    }
}
