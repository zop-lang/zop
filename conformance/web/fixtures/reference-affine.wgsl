@group(0) @binding(0)
var<storage, read> input_values: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output_values: array<f32>;

@compute @workgroup_size(64)
fn affine(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    output_values[index] = input_values[index] * 1.25 + 0.5;
}
