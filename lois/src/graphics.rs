use std::ops::Range;

use crate::{
    backend::Backend,
    batch::{TextureBatch, TextureBatchOptions},
    commons::{Color, Point, Rect, ViewSize},
    quad::Quad,
    texture::{Texture, TextureLoadOptions, TextureQuery, TextureUsage},
};

pub struct Graphics<B>
where
    B: Backend,
{
    backend: B,
    viewport_size: ViewSize,
    commands: Vec<DrawCommand>,
    quads: Vec<Quad>,
}

#[derive(Debug)]
pub enum DrawCommand {
    DrawTextureBatch(DrawTextureBatchCommand),
    Clear(ClearCommand),
}

#[derive(Debug)]
pub struct ClearCommand {
    pub target: Option<Texture>,
    pub color: Color,
}

#[derive(Debug)]
pub struct DrawTextureBatchCommand {
    pub texture: Texture,
    pub target: Option<Texture>,
    pub range: Range<usize>,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DrawOptions {
    pub src_rect: Option<Rect>,
    pub dest_rect: Option<Rect>,
    pub rotation_center: Option<Point>,
    pub rotation_angle: f32,
}

impl<B> Graphics<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Self {
        let viewport_size = backend.viewport();
        let commands = Vec::with_capacity(1000);
        let quads = Vec::with_capacity(10000);
        Self {
            viewport_size,
            commands,
            quads,
            backend,
        }
    }

    pub fn new_batch<'a>(
        &'a mut self,
        options: TextureBatchOptions,
    ) -> Result<TextureBatch<'a>, String> {
        self.push_command_if_necessary(options);
        let view_size = self.try_get_batch_view_size(options)?;
        let TextureQuery {
            size: texture_size, ..
        } = self
            .backend
            .query_texture(options.texture)
            .ok_or(String::from("Texture not found."))?;
        let command = self.commands.last_mut().unwrap();
        match command {
            DrawCommand::DrawTextureBatch(command) => {
                return Ok(TextureBatch::new(
                    view_size,
                    texture_size,
                    &mut self.quads,
                    command,
                ));
            }
            _ => panic!("Last command at command queue should be a DrawTextureBatch one."),
        }
    }

    pub fn clear(&mut self, color: Color, target: Option<Texture>) {
        self.commands
            .push(DrawCommand::Clear(ClearCommand { target, color }));
    }

    pub fn present(&mut self) {
        self.backend.present(&self.commands, &self.quads);
        self.commands.clear();
        self.quads.clear();
    }

    pub fn load_texture(&mut self, options: TextureLoadOptions) -> Result<Texture, String> {
        self.backend.load_texture(options)
    }

    pub fn unload_texture(&mut self, texture: Texture) {
        self.backend.unload_texture(texture);
    }

    pub fn query_texture(&self, texture: Texture) -> Option<TextureQuery> {
        self.backend.query_texture(texture)
    }

    pub fn resize_viewport(&mut self, new_size: ViewSize) {
        self.viewport_size = new_size;
        self.backend.resize_viewport(new_size);
    }

    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }

    fn push_command_if_necessary(&mut self, options: TextureBatchOptions) {
        match self.commands.last_mut() {
            Some(command) => match command {
                DrawCommand::DrawTextureBatch(DrawTextureBatchCommand {
                    texture, target, ..
                }) if *texture == options.texture && *target == options.target => return,
                _ => {}
            }
            _ => {}
        }

        self.commands
            .push(DrawCommand::DrawTextureBatch(DrawTextureBatchCommand {
                texture: options.texture,
                target: options.target,
                range: self.quads.len()..self.quads.len(),
            }));
    }

    fn try_get_batch_view_size(&self, options: TextureBatchOptions) -> Result<ViewSize, String> {
        if let Some(target) = options.target {
            if target == options.texture {
                return Err(String::from(
                    "A batch's texture cannot be the same as it's target.",
                ));
            }
            let target_query = self
                .query_texture(target)
                .ok_or(String::from("Target texture not found"))?;
            if target_query.usage != TextureUsage::RenderTarget {
                return Err(String::from(
                    "Target texture is not usable as RenderTarget.",
                ));
            }
            return Ok(target_query.size);
        } else {
            return Ok(self.viewport_size);
        }
    }
}
