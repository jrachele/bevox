#import bevy_sprite::mesh2d_view_bindings
#import "shaders/player.wgsl"
#import "shaders/voxel.wgsl"

#import "shaders/blocks/sand.wgsl"
#import "shaders/blocks/water.wgsl"

@group(1) @binding(0)
var<storage, read_write> voxel_double_grid: VoxelGrid;

const VOXEL_TYPE_SAND = 0u;
const VOXEL_TYPE_WATER = 1u;

@compute @workgroup_size(8, 8, 8)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = vec3<i32>(invocation_id);
    handle_voxel_physics(index, voxel_grid.voxels[get_index(index)]);

    // TODO: Move brush manipulation to a separate shader
    var selected = vec3<i32>(voxel_grid.selected);
    if ((player_data.mouse_click & 1u) == 1u) {
        // Left click
        set_voxel(selected, EMPTY_VOXEL);
    }
    if (((player_data.mouse_click >> 1u) & 1u) == 1u) {
        // Middle click
    }
    if (((player_data.mouse_click >> 2u) & 1u) == 1u) {
        // Right click
        var normal = vec3<i32>(voxel_grid.normal);
        selected = selected + normal;
        // Black sand
        set_voxel(selected, 1u << 9u);

        // Black solid
        // set_voxel(selected, 1u);
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
            handle_water(index);
        }
        default {}
    }
}
fn set_voxel(index: vec3<i32>, voxel: u32) -> bool {
    if (is_out_of_bounds(index)) {
        return false;
    }

    voxel_double_grid.voxels[get_index(index)] = voxel;
    return true;
}

fn get_voxel(index: vec3<i32>, voxel: ptr<function, u32>) -> bool {
    if (is_out_of_bounds(index)) {
        return false;
    }

    *voxel = voxel_grid.voxels[get_index(index)];
    return true;
}
