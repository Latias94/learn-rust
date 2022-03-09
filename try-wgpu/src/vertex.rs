/// Pod indicates that our Vertex is "Plain Old Data", and thus can be interpreted as a &[u8]
/// Zeroable indicates that we can use std::mem::zeroed(). We can modify our Vertex struct to derive these methods.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

impl Vertex {
    // Rust sees the result of vertex_attr_array is a temporary value, so a tweak is required to return it from a function.
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        // 返回值一大段代码可以用这个宏来代替
        let _with_macro = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        };

        wgpu::VertexBufferLayout {
            // The array_stride defines how wide a vertex is. When the shader goes to read the next vertex,
            // it will skip over array_stride number of bytes. In our case, array_stride will probably be 24 bytes.
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // step_mode tells the pipeline how often it should move to the next vertex.
            // This seems redundant in our case, but we can specify wgpu::VertexStepMode::Instance
            // if we only want to change vertices when we start drawing a new instance.
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    // This defines the offset in bytes until the attribute starts.
                    // For the first attribute the offset is usually zero. For any later attributes,
                    // the offset is the sum over size_of of the previous attributes' data.
                    offset: 0,
                    // This tells the shader what location to store this attribute at.
                    // For example [[location(0)]] x: vec3<f32> in the vertex shader would correspond to
                    // the position field of the Vertex struct, while [[location(1)]] x: vec3<f32> would be the color field.
                    shader_location: 0,
                    // format tells the shader the shape of the attribute. Float32x3 corresponds to vec3<f32> in shader code.
                    // The max value we can store in an attribute is Float32x4 (Uint32x4, and Sint32x4 work as well).
                    // We'll keep this in mind for when we have to store things that are bigger than Float32x4.
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
