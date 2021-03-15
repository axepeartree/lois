use crate::{
    commons::ViewportSize,
    graphics::{DrawOptions, DrawTextureBatchCommand},
    quad::Quad,
    texture::Texture,
};

use super::quad::QuadArrayVec;

pub struct TextureBatch<'a> {
    viewport_size: ViewportSize,
    quads: &'a mut QuadArrayVec,
    command: &'a mut DrawTextureBatchCommand,
    width: u32,
    height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct TextureBatchOptions {
    pub texture: Texture,
    pub target: Option<Texture>,
}

impl<'a> TextureBatch<'a> {
    pub(crate) fn new(
        viewport_size: ViewportSize,
        width: u32,
        height: u32,
        quads: &'a mut QuadArrayVec,
        command: &'a mut DrawTextureBatchCommand,
    ) -> Self {
        Self {
            viewport_size,
            quads,
            command,
            width,
            height,
        }
    }

    pub fn draw(&mut self, options: DrawOptions) {
        let quad = Quad::try_new_in_viewport(
            self.viewport_size,
            (self.width, self.height),
            options.src_rect,
            options.dest_rect,
            options.rotation_center,
            options.rotation_angle,
        );
        match quad {
            Some(quad) => {
                self.quads.push(quad);
                self.command.range.end += 1;
            }
            _ => {}
        }
    }
}
