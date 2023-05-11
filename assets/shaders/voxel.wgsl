@group(0) @binding(0)
var<storage, read_write> voxel_grid: VoxelGrid;

struct VoxelGrid {
    dim: u32,
    pos: vec3<f32>,
    voxels: array<u32>
}

const VOXEL_SIZE: f32 = 1.0;

fn get_index(index: vec3<i32>) -> u32 {
    let dim = i32(voxel_grid.dim);
    return u32((index.x * dim * dim) + (index.y * dim) + index.z);
}
