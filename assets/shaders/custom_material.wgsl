#import bevy_pbr::mesh_view_bindings

struct CustomMaterial {
    color: vec4<f32>,
    secondary: vec4<f32>
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput
) -> @location(0) vec4<f32> {

    // (0, 0, 0) -> material.color
    // (1, 1, 1) -> material.secondary

    var x = in.world_position.x;
    var y = in.world_position.z;
    var z = in.world_position.y;
    var t = (x + y + z) / 3.0;

    var s = (0.5 * (cos(t + globals.time) + 1.0));

    var new_vec = (((1.0 - s) * material.color) / 2.0) + ((s * material.secondary) / 2.0);
    new_vec.w = 1.0;

    return new_vec;
}
