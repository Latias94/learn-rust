use winit::event::WindowEvent;
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

        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        // submit will accept anything that implements IntoIter
        // tell wgpu to finish the command buffer, and to submit it to the gpu's render queue.
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
