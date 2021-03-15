use std::collections::HashMap;
use lois::{
    backend::Backend,
    commons::ViewportSize,
    graphics::DrawCommand,
    quad::Quad,
    texture::{Texture, TextureFormat, TextureLoadOptions, TextureQuery, TextureUsage},
};

pub struct BackendBlank {
    viewport_size: ViewportSize,
    textures: HashMap<u32, TextureBlank>,
    next_texture: Texture,
}

struct TextureBlank {
    name: Option<String>,
    width: u32,
    height: u32,
    usage: TextureUsage,
    format: TextureFormat,
}

impl BackendBlank {
    pub fn new() -> Self {
        Self {
            viewport_size: ViewportSize { width: 860, height: 640 },
            next_texture: Texture::new(0),
            textures: HashMap::with_capacity(100),
        }
    }
}

impl Backend for BackendBlank {
    fn present(&mut self, _commands: &[DrawCommand], _quads: &[Quad]) {}

    fn load_texture(&mut self, options: TextureLoadOptions) -> Result<Texture, String> {
        self.textures.insert(
            self.next_texture.id(),
            TextureBlank {
                name: Some(options.name.unwrap_or("Untitled").to_string()),
                format: options.format,
                width: options.width,
                height: options.height,
                usage: options.usage,
            },
        );
        let texture = self.next_texture;
        self.next_texture = Texture::new(self.next_texture.id() + 1);
        Ok(texture)
    }

    fn unload_texture(&mut self, texture: Texture) {
        self.textures.remove(&texture.id());
    }

    fn query_texture(&self, texture: Texture) -> Option<TextureQuery> {
        let texture = self.textures.get(&texture.id())?;
        Some(TextureQuery {
            name: texture.name.as_ref().map(|s| s.as_str()),
            format: texture.format,
            usage: texture.usage,
            width: texture.width,
            height: texture.height,
        })
    }

    fn resize_viewport(&mut self, new_size: ViewportSize) {
        self.viewport_size = new_size;
    }

    fn viewport(&self) -> ViewportSize {
        self.viewport_size
    }
}
