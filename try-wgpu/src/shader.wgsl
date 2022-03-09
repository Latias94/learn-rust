// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
};

// [[stage(vertex)]] to mark this function as a valid entry point for a vertex shader. 
[[stage(vertex)]]
fn vs_main(
    // // We expect a u32 called in_vertex_index which gets its value from [[builtin(vertex_index)]]
    // [[builtin(vertex_index)]] in_vertex_index: u32
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

// The [[location(0)]] bit tells WGPU to store the vec4 value returned by this function in the first color target. 
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}