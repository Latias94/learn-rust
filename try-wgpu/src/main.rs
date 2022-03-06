use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;
fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window));

    // create a window, and keep it open until the user closes it, or presses escape
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
            // We want State to have priority over main(). Doing that (and previous changes) should have your loop looking like this.
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    /// Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
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
        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    /// If we want to support resizing in our application, we're going to need to reconfigure the
    /// surface everytime the window's size changes. That's the reason we stored the physical size
    /// and the config used to configure the surface. With all of these, the resize method is very simple.
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// input() returns a bool to indicate whether an event has been fully processed.
    /// If the method returns true, the main loop won't process the event any further.
    fn input(&mut self, event: &WindowEvent) -> bool {
        // We're just going to return false for now because we don't have any events we want to capture.
        false
    }

    fn update(&mut self) {
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // The get_current_texture function will wait for the surface to provide a new SurfaceTexture that we will render to.
        // We'll store this in output for later.
        let output = self.surface.get_current_texture()?;
        // This line creates a TextureView with default settings.
        // We need to do this because we want to control how the render code interacts with the texture.
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // We also need to create a CommandEncoder to create the actual commands to send to the gpu.
        // Most modern graphics frameworks expect commands to be stored in a command buffer before being sent to the gpu.
        // The encoder builds a command buffer that we can then send to the gpu.
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
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
