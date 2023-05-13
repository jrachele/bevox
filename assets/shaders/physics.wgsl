#import bevy_sprite::mesh2d_view_bindings
#import "shaders/player.wgsl"
#import "shaders/physics_handler.wgsl"
#import "shaders/voxel.wgsl"

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
        set_voxel(selected, 1u << 9u);
    }
}
