use crate::{commons::ViewSize, graphics::DrawCommand, quad::Quad, texture::{Texture, TextureLoadOptions, TextureQuery}};

pub trait Backend {
    fn present(&mut self, commands: &[DrawCommand], quads: &[Quad]);
    fn load_texture(&mut self, options: TextureLoadOptions) -> Result<Texture, String>;
    fn unload_texture(&mut self, texture: Texture);
    fn query_texture(&self, texture: Texture) -> Option<TextureQuery>;
    fn resize_viewport(&mut self, new_size: ViewSize);
    fn viewport(&self) -> ViewSize;
}
