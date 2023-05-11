#import bevy_sprite::mesh2d_view_bindings
#import "shaders/physics_handler.wgsl"
#import "shaders/voxel.wgsl"

@compute @workgroup_size(8, 8, 8)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = vec3<i32>(invocation_id);
    handle_voxel_physics(index, voxel_grid.voxels[get_index(index)]);
}
