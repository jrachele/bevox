#import "shaders/blocks/sand.wgsl"
#import "shaders/blocks/water.wgsl"

const VOXEL_TYPE_SAND = 0u;
const VOXEL_TYPE_WATER = 1u;

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
