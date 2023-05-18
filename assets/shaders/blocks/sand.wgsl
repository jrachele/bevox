fn handle_sand(index: vec3<i32>) {
    if (out_of_bounds(index)) {
        return;
    }

    let current_voxel = voxel_grid.voxels[get_index(index)];
    if (current_voxel == EMPTY_VOXEL) {
        // voxel_grid_out.voxels[get_index(index)] = current_voxel;
        // storageBarrier();
        return;
    }

    let below_block_index = vec3<i32>(index.x, index.y - 1, index.z);

    if (!out_of_bounds(below_block_index)) {
        let below_block = voxel_grid.voxels[get_index(below_block_index)];
        // Ensure we have no write conflicts
        if (voxel_grid_out.voxels[get_index(below_block_index)] == EMPTY_VOXEL) {
            voxel_grid_out.voxels[get_index(index)] = EMPTY_VOXEL;
            voxel_grid_out.voxels[get_index(below_block_index)] = current_voxel;
            // storageBarrier();
            return;
        }
        // if (voxel_grid_out.voxels[get_index(below_block_index)] == EMPTY_VOXEL) {
        //     voxel_grid_out.voxels[get_index(index)] = EMPTY_VOXEL;
        //     voxel_grid_out.voxels[get_index(below_block_index)] = current_voxel;
        //     // storageBarrier();
        //     return;
        // }

        // Let's try to move to a different spot in the xz plane at y-1
        // use index.x and index.y as hash inputs for the randomizer
        for (var i = -1; i <= 1; i++) {
            for (var j = -1; j <= 1; j++) {
                let x_offset = i;
                let z_offset = j;
                let side_block_index = vec3<i32>(index.x + x_offset, index.y - 1, index.z + z_offset);
                if (!out_of_bounds(side_block_index)) {
                    let side_block = voxel_grid.voxels[get_index(side_block_index)];
                    // Ensure no conflicts
                    if (voxel_grid_out.voxels[get_index(side_block_index)] == EMPTY_VOXEL) {
                        voxel_grid_out.voxels[get_index(index)] = EMPTY_VOXEL;
                        voxel_grid_out.voxels[get_index(side_block_index)] = current_voxel;
                        storageBarrier();
                        return;
                    }
                }
            }
        }
        // let x_offset = random_int(u32(index.x), -1, 1);
        // let z_offset = random_int(u32(index.y), -1, 1);
        // let side_block_index = vec3<i32>(index.x + x_offset, index.y - 1, index.z + z_offset);
        // if (!out_of_bounds(side_block_index)) {
        //     let side_block = voxel_grid.voxels[get_index(side_block_index)];
        //     // Ensure no conflicts
        //     if (voxel_grid_out.voxels[get_index(side_block_index)] == EMPTY_VOXEL) {
        //         voxel_grid_out.voxels[get_index(index)] = EMPTY_VOXEL;
        //         voxel_grid_out.voxels[get_index(side_block_index)] = current_voxel;
        //         storageBarrier();
        //         return;
        //     }
        // }
    }

    // If we weren't able to move the block before, store its current state
    voxel_grid_out.voxels[get_index(index)] = current_voxel;
    storageBarrier();
}
