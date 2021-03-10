mod commons;
mod pipeline;

pub use self::pipeline::{DrawTextureOptions, TextureDescriptor, TextureHandle};
pub use commons::{Color, Rect};
use pipeline::Pipeline2D;
use raw_window_handle::HasRawWindowHandle;

pub struct GraphicsState {
    viewport_size: ViewportSize,
    _instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    pipeline: Pipeline2D,
}

#[derive(Copy, Clone, Debug)]
pub struct ViewportSize {
    width: u32,
    height: u32,
}

impl GraphicsState {
    /// Creates an instance of the GraphicsState.
    /// `window` must be a valid object.
    pub async unsafe fn new(
        window: &impl HasRawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::METAL);

        let surface = instance.create_surface(window);

        let size = ViewportSize { width, height };

        let (device, queue) = {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    power_preference: wgpu::PowerPreference::HighPerformance,
                })
                .await
                .ok_or(String::from("Unable to request a suitable wgpu::Adapter."))?;

            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("Device"),
                        features: wgpu::Features::empty(),
                        limits: Default::default(),
                    },
                    None,
                )
                .await
                .map_err(|err| err.to_string())?
        };

        let swap_chain = create_swap_chain(&device, &surface, size.width, size.height);

        let renderer = Pipeline2D::new(&device, size);

        Ok(Self {
            viewport_size: size,
            _instance: instance,
            surface,
            device,
            queue,
            swap_chain,
            pipeline: renderer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport_size = ViewportSize { width, height };
        self.swap_chain = create_swap_chain(
            &self.device,
            &self.surface,
            self.viewport_size.width,
            self.viewport_size.height,
        );
        self.pipeline
            .update_uniforms(&self.queue, self.viewport_size);
    }

    pub fn load_texture(&mut self, descriptor: TextureDescriptor) -> Result<TextureHandle, String> {
        self.pipeline
            .load_texture(&self.device, &self.queue, descriptor)
    }

    pub fn clear(&mut self, color: Color) {
        self.pipeline.clear(color);
    }

    pub fn draw_texture(
        &mut self,
        texture: TextureHandle,
        options: DrawTextureOptions,
    ) -> Result<(), String> {
        self.pipeline
            .draw_texture(&self.device, texture, options, self.viewport_size)
    }

    pub fn render(&mut self) -> Result<(), String> {
        let current_frame = match self.swap_chain.get_current_frame() {
            Ok(current_frame) => current_frame,
            Err(wgpu::SwapChainError::OutOfMemory) => {
                self.resize(self.viewport_size.width, self.viewport_size.height);
                return Ok(());
            }
            Err(err) => return Err(format!("{:?}", err)),
        };

        self.pipeline
            .render(&self.device, &self.queue, &current_frame)
    }
}

fn create_swap_chain(
    device: &wgpu::Device,
    surface: &wgpu::Surface,
    width: u32,
    height: u32,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            present_mode: wgpu::PresentMode::Fifo,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
        },
    )
}
