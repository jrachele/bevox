@group(0) @binding(0)
var<storage, read_write> voxel_grid: VoxelGrid;

struct VoxelGrid {
    dim: u32,
    pos: vec3<f32>,
    selected: vec3<f32>,
    normal: vec3<f32>,
    voxels: array<u32>
}

const VOXEL_SIZE: f32 = 1.0;
const EMPTY_VOXEL: u32 = 0u;

fn get_index(index: vec3<i32>) -> u32 {
    let dim = i32(voxel_grid.dim);
    return u32((index.x * dim * dim) + (index.y * dim) + index.z);
}

fn is_out_of_bounds(index: vec3<i32>) -> bool {
    let dim = i32(voxel_grid.dim);
    return index.x <= 0 || index.x >= dim ||
        index.y <= 0 || index.y >= dim ||
        index.z <= 0 || index.z >= dim;
}

fn get_voxel_color(voxel_data: u32) -> vec3<f32> {
    let mask_5 = 31u;
    let mask_6 = 63u;
    let r_offset = 32u - 5u;
    let g_offset = 32u - 11u;
    let b_offset = 32u - 16u;
    let r: u32 = (voxel_data & (mask_5 << r_offset)) >> r_offset;
    let g: u32 = (voxel_data & (mask_6 << g_offset)) >> g_offset;
    let b: u32 = (voxel_data & (mask_5 << b_offset)) >> b_offset;
    return vec3<f32>(f32(r) / f32(mask_5), f32(g) / f32(mask_6), f32(b) / f32(mask_5));
}

fn set_voxel_color(voxel_data: u32, color: vec3<f32>) -> u32 {
    let r: u32 = u32(color.x * 31.0);
    let g: u32 = u32(color.y * 63.0);
    let b: u32 = u32(color.z * 31.0);
    let r_offset = 32u - 5u;
    let g_offset = 32u - 11u;
    let b_offset = 32u - 16u;
    var voxel = voxel_data;
    voxel |= (r << r_offset);
    voxel |= (g << g_offset);
    voxel |= (b << b_offset);
    return voxel;
}

fn get_voxel_type(voxel_data: u32) -> u32 {
    return voxel_data & 255u;
}

fn set_voxel_type(voxel_data: u32, voxel_type: u32) -> u32 {
    var voxel = voxel_data;
    voxel >>= 8u;
    voxel <<= 8u;
    voxel |= voxel_type & 255u;
    return voxel;

}

const colors = array<vec3<f32>, 1>(vec3<f32>(0.4, 0.3, 0.1));

fn get_default_color(voxel_data: u32) -> vec3<f32> {
    let voxel_type = get_voxel_type(voxel_data);
    switch (voxel_type) {
        case 0u: {
            return colors[0u];
        }
        default {
        }
    }
    return vec3<f32>(1.0, 0.0, 0.0);
}

fn hash(value: u32) -> u32 {
  var hashedValue: u32 = value;
  hashedValue ^= hashedValue >> 16u;
  hashedValue *= 0x85ebca6bu;
  hashedValue ^= hashedValue >> 13u;
  hashedValue *= 0xc2b2ae35u;
  hashedValue ^= hashedValue >> 16u;
  return hashedValue;
}

fn randomFloat(value: u32) -> f32 {
  var hashedValue: u32 = hash(value);
  return f32(hashedValue) / 4294967296.0;  // Divide by 2^32
}
