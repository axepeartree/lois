use crate::{commons::ViewportSize, graphics::DrawCommand, quad::Quad, texture::{Texture, TextureLoadOptions, TextureQuery}};

pub mod blank;

#[cfg(any(feature = "default", feature = "wgpu"))]
pub mod wgpu;

pub trait Backend {
    fn present(&mut self, commands: &[DrawCommand], quads: &[Quad]);
    fn load_texture(&mut self, options: TextureLoadOptions) -> Result<Texture, String>;
    fn unload_texture(&mut self, texture: Texture);
    fn query_texture(&self, texture: Texture) -> Option<TextureQuery>;
    fn resize_viewport(&mut self, new_size: ViewportSize);
    fn viewport(&mut self) -> ViewportSize;
}
