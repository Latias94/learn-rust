use crate::camera::{Camera, CameraController, CameraUniform};
use crate::texture;
use crate::vertex::{Vertex, INDICES, VERTICES};
use wgpu::util::DeviceExt;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

pub struct State {
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    #[allow(dead_code)]
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: texture::Texture,
    pub cartoon_bind_group: wgpu::BindGroup,
    pub cartoon_texture: texture::Texture,
    pub is_space_pressed: bool,
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_controller: CameraController,
}

impl State {
    /// Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        // let instance = wgpu::Instance::new(wgpu::Backends::all()); // 用 vulkan 有报错，暂时用 dx12
        let instance = wgpu::Instance::new(wgpu::Backends::DX12);
        // The surface is the part of the window that we draw to. We need it to draw directly to the screen.
        let surface = unsafe { instance.create_surface(window) };
        // The adapter is a handle to our actual graphics card.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), // LowPower or HighPerformance
                compatible_surface: Some(&surface), // tells wgpu to find an adapter that can present to the supplied surface.
                force_fallback_adapter: false, // forces wgpu to pick an adapter that will work on all hardware.
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(), // The limits field describes the limit of certain types of resources that we can create.
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        // This will define how the surface creates its underlying SurfaceTextures.
        // We will talk about SurfaceTexture when we get to the render function.
        let config = wgpu::SurfaceConfiguration {
            // The usage field describes how SurfaceTextures will be used.
            // RENDER_ATTACHMENT specifies that the textures will be used to write to the screen (we'll talk about more TextureUsages later).
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // The format defines how SurfaceTextures will be stored on the gpu.
            // Different displays prefer different formats. We use surface.get_preferred_format(&adapter)
            // to figure out the best format to use based on the display you're using.
            format: surface.get_preferred_format(&adapter).unwrap(),
            // width and height are the width and the height in pixels of a SurfaceTexture.
            // This should usually be the width and the height of the window.
            width: size.width,
            height: size.height,
            // present_mode uses wgpu::PresentMode enum which determines how to sync the surface with the display.
            //  The option we picked, FIFO, will cap the display rate at the displays framerate. This is essentially VSync.
            // This is also the most optimal mode on mobile. There are other options and you can see all of them in the docs
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        // BindGroup 描述了一组资源以及它们如何被着色器访问。我们使用 BindGroupLayout 创建一个 BindGroup。
        // 我们的 texture_bind_group_layout 有两个条目：一个是绑定 0 的采样纹理，另一个是绑定 1 的采样器。
        // 这两个绑定只对 FRAGMENT 所指定的片段着色器可见。这个字段的可能值是 NONE、VERTEX、FRAGMENT 或 COMPUTE 的任意位数组合。
        // 大多数情况下，我们只对纹理和采样器使用 FRAGMENT，但知道还有什么可用的也很好。
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            // SamplerBindingType::Comparison is only for TextureSampleType::Depth
                            // SamplerBindingType::Filtering if the sample_type of the texture is:
                            //     TextureSampleType::Float { filterable: true }
                            // Otherwise you'll get an error.
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        // BindGroup 是 BindGroupLayout 的一个更具体的声明。它们分开的原因是它允许我们动态交换 BindGroup，
        // 只要它们都共享同一个 BindGroupLayout。我们创建的每个纹理和采样器都需要被添加到一个 BindGroup 中。
        // 为了我们的目的，我们将为每个纹理创建一个新的绑定组。
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let cartoon_bytes = include_bytes!("happy-tree-cartoon.png");
        let cartoon_texture =
            texture::Texture::from_bytes(&device, &queue, cartoon_bytes, "happy-tree-cartoon.png")
                .unwrap();
        let cartoon_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cartoon_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cartoon_texture.sampler),
                },
            ],
            label: Some("cartoon_bind_group"),
        });

        let clear_color = wgpu::Color::BLACK;
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        // 也可以用宏：
        // let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        // 下面的 API 对于熟悉 OpenGL, Dx 接口的应该就很熟悉了
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1. 指定 shader 对应着色器的函数入口名
                buffers: &[Vertex::desc()], // 2. 要传递给顶点着色器的顶点类型
            },
            fragment: Some(wgpu::FragmentState {
                // 3. 片元着色器是可选的，所以这里用 Some 包裹
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    // 4. tells wgpu what color outputs it should set up
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            // describes how to interpret our vertices when converting them into triangles.
            primitive: wgpu::PrimitiveState {
                // 1. Using PrimitiveTopology::TriangleList means that each three vertices will correspond to one triangle.
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // 2. The `front_face` and `cull_mode` fields tell wgpu how to determine whether a given triangle is facing forward or not.
                front_face: wgpu::FrontFace::Ccw, // 根据摄像机的观察视角，将顶点顺序为逆时针方向的三角形看作正面朝向，而把顺时针绕序的三角形当作背面朝向。
                cull_mode: Some(wgpu::Face::Back), // 剔除背面朝向的三角形
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            // 1. We're not using a depth/stencil buffer currently, so we leave depth_stencil as None.
            // This will change later.
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                // 2. count determines how many samples the pipeline will use.
                // Multisampling is a complex topic, so we won't get into it here.
                count: 1,
                // 3. mask specifies which samples should be active. In this case we are using all of them.
                mask: !0,
                // 4. alpha_to_coverage_enabled has to do with anti-aliasing.
                // We're not covering anti-aliasing here, so we'll leave this as false now.
                alpha_to_coverage_enabled: false,
            },
            // 5. multiview indicates how many array layers the render attachments can have.
            // We won't be rendering to array textures so we can set this to None.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES), // using bytemuck to cast our VERTICES as a &[u8]
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_vertices = VERTICES.len() as u32;
        let num_indices = INDICES.len() as u32;
        let is_space_pressed = false;
        let camera_controller = CameraController::new(0.2);
        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            cartoon_bind_group,
            cartoon_texture,
            is_space_pressed,
        }
    }

    /// If we want to support resizing in our application, we're going to need to reconfigure the
    /// surface everytime the window's size changes. That's the reason we stored the physical size
    /// and the config used to configure the surface. With all of these, the resize method is very simple.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// input() returns a bool to indicate whether an event has been fully processed.
    /// If the method returns true, the main loop won't process the event any further.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // clear color 随着鼠标位置变化
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if keycode == &VirtualKeyCode::Space {
                    self.is_space_pressed = *state == ElementState::Pressed;
                }
                self.camera_controller.process_events(event);

                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // 1. 我们可以创建一个单独的缓冲区，并把它的内容复制到我们的 Camera_buffer。这个新的缓冲区被称为暂存缓冲区 staging buffer。
        // 这种方法通常是这样做的，因为它允许主缓冲区（在这种情况下是 camera_buffer）的内容只被 gpu 访问。
        // 2. 我们可以在缓冲区本身调用映射方法的 map_read_async 和 map_write_async。这些方法允许我们直接访问缓冲区的内容，
        // 但需要我们处理这些方法的异步问题，这也需要我们的缓冲区使用 BufferUsages::MAP_READ 和 / 或 BufferUsages::MAP_WRITE。
        // 我们在这里就不说了，如果你想了解更多，可以查看 Wgpu without a window 教程。
        // 3. 我们可以在 queue 上使用 write_buffer。
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // The get_current_texture function will wait for the surface to provide a new SurfaceTexture that we will render to.
        // We'll store this in output for later.
        let output = self.surface.get_current_texture()?;
        // This line creates a TextureView with default settings.
        // We need to do this because we want to control how the render code interacts with the texture.
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // We also need to create a CommandEncoder to create the actual commands to send to the gpu.
        // Most modern graphics frameworks expect commands to be stored in a command buffer before being sent to the gpu.
        // The encoder builds a command buffer that we can then send to the gpu.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // clearing the screen
        // We need to use the encoder to create a RenderPass.
        // The RenderPass has all the methods for the actual drawing.
        // 这个大括号作用域是为了 begin_render_pass() 借用了 encoder (&mut self)，要等释放了可变借用后，才能调用 encoder.finish()。
        // 也可以 drop(render_pass)。
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // The color_attachments describe where we are going to draw our color to.
                // We use the TextureView we created earlier to make sure that we render to the screen.
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    // The resolve_target is the texture that will receive the resolved output.
                    // This will be the same as view unless multisampling is enabled.
                    // We don't need to specify this, so we leave it as None.
                    resolve_target: None,
                    // The ops field takes a wgpu::Operations object.
                    // This tells wgpu what to do with the colors on the screen (specified by view).
                    ops: wgpu::Operations {
                        // The load field tells wgpu how to handle colors stored from the previous frame.
                        // Currently, we are clearing the screen with a bluish color.
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        // The store field tells wgpu whether we want to store the rendered results to
                        // the Texture behind our TextureView (in this case it's the SurfaceTexture).
                        store: true,
                    },
                }],
                // We'll use depth_stencil_attachment later, but we'll set it to None for now.
                depth_stencil_attachment: None,
            });
            let data = (&self.vertex_buffer, &self.index_buffer, self.num_indices);

            // 前面创建了 render pipeline，这里要给 pass 设置上
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(
                0,
                if !self.is_space_pressed {
                    &self.diffuse_bind_group
                } else {
                    &self.cartoon_bind_group
                },
                &[],
            );
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            // One more thing: we need to actually set the vertex buffer in the render method otherwise our program will crash.
            // set_vertex_buffer takes two parameters. The first is what buffer slot to use for this vertex buffer.
            // You can have multiple vertex buffers set at a time.
            // The second parameter is the slice of the buffer to use. You can store as many objects in a buffer as your hardware allows,
            // so slice allows us to specify which portion of the buffer to use. We use .. to specify the entire buffer.
            render_pass.set_vertex_buffer(0, data.0.slice(..));
            render_pass.set_index_buffer(data.1.slice(..), wgpu::IndexFormat::Uint16);
            // When using an index buffer, you need to use draw_indexed. The draw method ignores the index buffer.
            // Also make sure you use the number of indices (num_indices), not vertices as your model will either draw wrong,
            // or the method will panic because there are not enough indices.
            render_pass.draw_indexed(0..data.2, 0, 0..1);

            // We tell wgpu to draw something with 3 vertices, and 1 instance.
            // This is where [[builtin(vertex_index)]] comes from. 我们手动传了顶点进去
            // render_pass.draw(0..self.num_vertices, 0..1);
        }

        // submit will accept anything that implements IntoIter
        // tell wgpu to finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
