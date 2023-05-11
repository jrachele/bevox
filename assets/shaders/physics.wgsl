#import bevy_sprite::mesh2d_view_bindings
#import "shaders/player.wgsl"
#import "shaders/physics_handler.wgsl"
#import "shaders/voxel.wgsl"

@compute @workgroup_size(8, 8, 8)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = vec3<i32>(invocation_id);
    handle_voxel_physics(index, voxel_grid.voxels[get_index(index)]);

    // TODO: Move brush manipulation to a separate shader
    if ((player_data.mouse_click & 1u) == 1u) {
        // Left click
        let selected = voxel_grid.selected;
        voxel_grid.voxels[get_index(vec3<i32>(selected))] = 0u;
    }
    if (((player_data.mouse_click >> 1u) & 1u) == 1u) {
        // Middle click
        let selected = voxel_grid.selected;
        voxel_grid.voxels[get_index(vec3<i32>(selected))] = 0u;
    }
    if (((player_data.mouse_click >> 2u) & 1u) == 1u) {
        // Right click
        var selected = voxel_grid.selected;
        var normal = voxel_grid.normal;
        normal.y *= -1.0;
        selected = selected - normal;
        // TODO: Use sign of difference between camera position w1 w2 w3 -> x y z
        // to determine where the block should be placed given normals
        if (!is_out_of_bounds(vec3<i32>(selected))) {
            voxel_grid.voxels[get_index(vec3<i32>(selected))] = 1u;
        }
    }
}
