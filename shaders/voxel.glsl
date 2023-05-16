// Voxel types

const uint EMPTY_VOXEL = 0;
const uint VT_SAND = 0;
const uint VT_WATER = 1;

vec3 get_voxel_color(uint voxel_data) {
    uint mask_5 = 31u;
    uint mask_6 = 63u;
    uint r_offset = 32u - 5u;
    uint g_offset = 32u - 11u;
    uint b_offset = 32u - 16u;
    uint r = (voxel_data & (mask_5 << r_offset)) >> r_offset;
    uint g = (voxel_data & (mask_6 << g_offset)) >> g_offset;
    uint b = (voxel_data & (mask_5 << b_offset)) >> b_offset;
    return vec3(float(r) / float(mask_5), float(g) / float(mask_6), float(b) / float(mask_5));
}

uint set_voxel_color(uint voxel_data, vec3 color) {
    uint r = uint(color.x * 31.0);
    uint g = uint(color.y * 63.0);
    uint b = uint(color.z * 31.0);
    uint r_offset = 32u - 5u;
    uint g_offset = 32u - 11u;
    uint b_offset = 32u - 16u;
    uint voxel = voxel_data;
    voxel |= (r << r_offset);
    voxel |= (g << g_offset);
    voxel |= (b << b_offset);
    return voxel;
}

uint get_voxel_type(uint voxel_data) {
    return voxel_data & 255u;
}

uint set_voxel_type(uint voxel_data, uint voxel_type) {
    uint voxel = voxel_data;
    voxel >>= 8u;
    voxel <<= 8u;
    voxel |= voxel_type & 255u;
    return voxel;
}

int get_index(ivec3 index) {
    int dim = int(push_constants.dim);
    return (index.x * dim * dim) + (index.y * dim) + index.z;
}

bool out_of_bounds(ivec3 index) {
    int dim = int(push_constants.dim);
    return index.x < 0 || index.y < 0 || index.z < 0 || index.x >= dim || index.y >= dim || index.z >= dim;
}
