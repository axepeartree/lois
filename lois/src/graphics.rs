use std::ops::Range;

use crate::{
    backend::Backend,
    batch::{TextureBatch, TextureBatchOptions},
    commons::{Color, Point, Rect, ViewportSize},
    quad::QuadArrayVec,
    texture::{Texture, TextureLoadOptions, TextureQuery, TextureUsage},
};

pub struct Graphics<B>
where
    B: Backend,
{
    backend: B,
    viewport_size: ViewportSize,
    commands: Vec<DrawCommand>,
    quads: QuadArrayVec,
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
    pub fn new(mut backend: B) -> Self {
        let viewport_size = backend.viewport();
        let commands = Vec::with_capacity(1000);
        let quads = QuadArrayVec::new();
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
        let TextureQuery { width, height, .. } = self.query_texture(options.texture)
            .ok_or(String::from("Texture not found."))?;

        if let Some(target)= options.target {
            if target == options.texture {
                return Err(String::from("A batch's texture cannot be the same as it's target."))
            }
            let target_query = self.query_texture(target).ok_or(String::from("Target texture not found"))?;
            if target_query.usage != TextureUsage::RenderTarget {
                return Err(String::from("Target texture is not usable as RenderTarget."))
            }
        }

        let quads = &mut self.quads;
        let commands = &mut self.commands;
        let viewport_size = self.viewport_size;

        let is_last = if let Some(command) = commands.last_mut() {
            match command {
                DrawCommand::DrawTextureBatch(DrawTextureBatchCommand {
                    texture, target, ..
                }) if *texture == options.texture && *target == options.target => true,
                _ => false,
            }
        } else {
            false
        };

        if !is_last {
            commands.push(DrawCommand::DrawTextureBatch(DrawTextureBatchCommand {
                texture: options.texture,
                target: options.target,
                range: quads.next()..quads.next(),
            }));
        }

        let command = commands.last_mut().unwrap();
        match command {
            DrawCommand::DrawTextureBatch(command) => {
                return Ok(TextureBatch::new(viewport_size, width, height, quads, command));
            }
            _ => panic!(),
        }
    }

    pub fn clear(&mut self, target: Option<Texture>, color: Color) {
        self.commands
            .push(DrawCommand::Clear(ClearCommand { target, color }));
    }

    pub fn present(&mut self) {
        self.backend.present(&self.commands, self.quads.slice(..));
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

    pub fn resize_viewport(&mut self, new_size: ViewportSize) {
        self.viewport_size = new_size;
        self.backend.resize_viewport(new_size);
    }

    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }
}
