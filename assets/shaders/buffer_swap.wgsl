#import "shaders/voxel.wgsl"

@group(1) @binding(0)
var<storage, read_write> voxel_double_grid: VoxelGrid;

@compute @workgroup_size(8, 8, 8)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    var index = vec3<i32>(invocation_id);
    let flat_index = get_index(index);
    let voxel = voxel_double_grid.voxels[flat_index];
    voxel_grid.voxels[flat_index] = voxel;
}
