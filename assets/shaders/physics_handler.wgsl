#import "shaders/sand.wgsl"

fn handle_voxel_physics(index: vec3<i32>, voxel: u32) {
    if (voxel == EMPTY_VOXEL) {
        return;
    }

    switch get_voxel_type(voxel) {
        case 0u {
            handle_sand(index);
        }
        default {}
    }
}
