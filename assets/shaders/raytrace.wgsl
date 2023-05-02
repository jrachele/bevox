@group(0) @binding(0)
var<storage, read> voxel_data: VoxelData;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;

struct VoxelData {
    pos: vec3<f32>,
    voxels: array<u32, 1000>
}


fn get_index(x: i32, y: i32, z: i32, n: i32) -> u32 {
    return u32((x * n * n) + (y * n) + z);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    for (var i: i32 = 0; i < 10; i++) {
        for (var j: i32 = 0; j < 10; j++) {
            for (var k: i32 = 0; k < 10; k++) {
                let voxel = voxel_data.voxels[get_index(i, j, k, 10)];

                let location = vec2<i32>(i32(voxel_data.pos.x) + i * 10, i32(voxel_data.pos.y) + j * 10);
                // storageBarrier();

                if (voxel != 0u) {
                    let color = vec4<f32>(1.0, 0.5, 1.0, 1.0);
                    textureStore(output_texture, location, color);
                }
            }
        }
    }
}
