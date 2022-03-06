// Vertex shader

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

// [[stage(vertex)]] to mark this function as a valid entry point for a vertex shader. 
[[stage(vertex)]]
fn vs_main(
    // We expect a u32 called in_vertex_index which gets its value from [[builtin(vertex_index)]]
    [[builtin(vertex_index)]] in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    // type casting
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

// Fragment shader

// The [[location(0)]] bit tells WGPU to store the vec4 value returned by this function in the first color target. 
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}