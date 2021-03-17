use crate::{commons::ViewSize, graphics::{DrawOptions, DrawTextureBatchCommand}, quad::Quad, texture::Texture};

pub struct TextureBatch<'a> {
    quads: &'a mut Vec<Quad>,
    command: &'a mut DrawTextureBatchCommand,
    target_size: ViewSize,
    texture_size: ViewSize,
}

#[derive(Copy, Clone, Debug)]
pub struct TextureBatchOptions {
    pub texture: Texture,
    pub target: Option<Texture>,
}

impl<'a> TextureBatch<'a> {
    pub(crate) fn new(
        target_size: ViewSize,
        texture_size: ViewSize,
        quads: &'a mut Vec<Quad>,
        command: &'a mut DrawTextureBatchCommand,
    ) -> Self {
        Self {
            quads,
            command,
            target_size,
            texture_size,
        }
    }

    pub fn draw(mut self, options: DrawOptions) -> Self {
        let quad = Quad::new(
            self.target_size,
            self.texture_size,
            options.src_rect,
            options.dest_rect,
            options.rotation_center,
            options.rotation_angle,
        );
        self.quads.push(quad);
        self.command.range.end += 1;
        self
    }
}

impl TextureBatchOptions {
    pub fn new(texture: Texture, target: Option<Texture>) -> Self {
        Self { texture, target }
    }
}