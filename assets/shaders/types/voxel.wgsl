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

fn out_of_bounds(index: vec3<i32>) -> bool {
    let dim = i32(voxel_grid.dim);
    return index.x < 0 || index.x >= dim ||
        index.y < 0 || index.y >= dim ||
        index.z < 0 || index.z >= dim;
}

// This may become useful sometime
fn out_of_invocation_bounds(invocation_id: vec3<u32>, invocation_id_local: vec3<u32>, index: vec3<i32>) -> bool {
    let boundary_bottom_left = vec3<i32>(invocation_id - invocation_id_local);
    let boundary_top_right = boundary_bottom_left + vec3<i32>(8, 8, 8);
    return index.x < boundary_bottom_left.x || index.y < boundary_bottom_left.y || index.z < boundary_bottom_left.z ||
        index.x >= boundary_top_right.x || index.y >= boundary_top_right.y || index.z >= boundary_top_right.z;
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

fn random_float(value: u32) -> f32 {
  var hashedValue: u32 = hash(value);
  return f32(hashedValue) / 4294967296.0;  // Divide by 2^32
}

fn random_int(value: u32, lower_bound: i32, upper_bound: i32) -> i32 {
    let f = random_float(value);
    let range = upper_bound - lower_bound + 1;
    let adjusted = i32(f * f32(f)) % range; // % just in case, 1 / 2^32 chance its exactly 1.0
    return adjusted + lower_bound;
}
