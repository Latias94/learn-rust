// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};

// 因为我们已经创建了一个新的绑定组，我们需要指定在着色器中使用哪一个。这个数字是由我们的 render_pipeline_layout 决定的。
// texture_bind_group_layout 被列在第一位，因此它是 group (0)，而 camera_bind_group 是第二位，因此它是 group (1)。
[[group(1), binding(0)]] // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

// [[stage(vertex)]] to mark this function as a valid entry point for a vertex shader. 
[[stage(vertex)]]
fn vs_main(
    // // We expect a u32 called in_vertex_index which gets its value from [[builtin(vertex_index)]]
    // [[builtin(vertex_index)]] in_vertex_index: u32
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    // 当涉及到矩阵时，乘法顺序很重要。向量在右边，矩阵在左边，按重要性排序
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

// The [[location(0)]] bit tells WGPU to store the vec4 value returned by this function in the first color target. 
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // 变量 t_diffuse 和 s_diffuse 就是所谓的 uniforms。我们将在摄像机部分更多地讨论 uniforms 问题。
    // 现在，我们需要知道的是，group () 与 set_bind_group () 中的第一个参数相对应，binding () 与我们创建 BindGroupLayout 和 BindGroup 时指定的绑定有关。
     return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
