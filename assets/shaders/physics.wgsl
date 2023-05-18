#import bevy_sprite::mesh2d_view_bindings
#import "shaders/player.wgsl"
#import "shaders/voxel.wgsl"

#import "shaders/blocks/sand.wgsl"
// #import "shaders/blocks/water.wgsl"

@group(1) @binding(0)
var<storage, read_write> voxel_grid_out: VoxelGrid;

const VOXEL_TYPE_SAND = 0u;
const VOXEL_TYPE_WATER = 1u;

@compute @workgroup_size(8, 8, 8)
// @compute @workgroup_size(1, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(local_invocation_id) invocation_id_local: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = vec3<i32>(invocation_id);
    handle_voxel_physics(index, voxel_grid.voxels[get_index(index)]);
    // for (var i = 0u; i < voxel_grid.dim; i++) {
    //     for (var j = 0u; j < voxel_grid.dim; j++) {
    //         for (var k = 0u; k < voxel_grid.dim; k++) {
    //             var index = vec3<i32>(vec3<u32>(i, j, k));
    //             handle_voxel_physics(index, voxel_grid.voxels[get_index(index)]);
    //         }
    //     }
    // }

    // TODO: Move brush manipulation to a separate shader
    var selected = vec3<i32>(voxel_grid.selected);
    if ((player_data.mouse_click & 1u) == 1u) {
        // Left click
        voxel_grid_out.voxels[get_index(selected)] = 0u;
    }
    if (((player_data.mouse_click >> 1u) & 1u) == 1u) {
        // Middle click
    }
    if (((player_data.mouse_click >> 2u) & 1u) == 1u) {
        // Right click
        var normal = vec3<i32>(voxel_grid.normal);
        selected = selected + normal;
        // Black sand
        voxel_grid_out.voxels[get_index(selected)] = 1u << 9u;
    }
}

fn handle_voxel_physics(index: vec3<i32>, voxel: u32) {
    if (voxel == EMPTY_VOXEL) {
        return;
    }

    switch get_voxel_type(voxel) {
        case 0u {
            handle_sand(index);
        }
        case 1u {
            // handle_water(index);
        }
        default {}
    }
}
