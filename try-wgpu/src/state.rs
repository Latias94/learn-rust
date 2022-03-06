use wgpu::{ShaderModule, TextureFormat};
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
    pub challenge_render_pipeline: wgpu::RenderPipeline,
    pub use_color: bool,
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
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        // 下面的 API 对于熟悉 OpenGL, Dx 接口的应该就很熟悉了
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1. 指定 shader 对应着色器的函数入口名
                buffers: &[],           // 2. 要传递给顶点着色器的顶点类型
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
                front_face: wgpu::FrontFace::Ccw, // 根据摄像机的观察视角，将顶点顺序为逆时针方向的三角形看作正面朝向，而把顺 时针绕序的三角形当作背面朝向。
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

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Challenge Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("challenge.wgsl").into()),
        });

        let challenge_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

        let use_color = true;

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
            challenge_render_pipeline,
            use_color,
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
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                self.use_color = *state == ElementState::Released;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {}

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
            // 前面创建了 render pipeline，这里要给 pass 设置上
            render_pass.set_pipeline(if self.use_color {
                &self.render_pipeline
            } else {
                &self.challenge_render_pipeline
            });
            // We tell wgpu to draw something with 3 vertices, and 1 instance.
            // This is where [[builtin(vertex_index)]] comes from. 我们手动传了顶点进去
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter
        // tell wgpu to finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}