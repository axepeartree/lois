use std::ops::RangeFull;

use crate::commons::{Point, Rect, ViewportSize};

#[derive(Debug)]
pub(crate) struct QuadArrayVec {
    quads: Vec<Quad>,
    next: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Quad {
    transform: [[f32; 4]; 4],
    src_rect: [f32; 4],
}

const QUADS_SLICE_ZEROED: &[Quad] = &[Quad::empty(); 1000];

const UNIT_QUAD: &[[f32; 2]] = &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

impl Quad {
    const fn empty() -> Self {
        Self {
            transform: [[0.0, 0.0, 0.0, 0.0]; 4],
            src_rect: [0.0, 0.0, 0.0, 0.0],
        }
    }

    #[allow(unused_variables)]
    pub fn try_new_in_viewport(
        viewport_size: ViewportSize,
        texture_size: (u32, u32),
        src_rect: Option<Rect>,
        dest_rect: Option<Rect>,
        rotation_center: Option<Point>,
        rotation_angle: f32,
    ) -> Option<Self> {
        // TODO: this is probably slow; we should benchmark and optimize...

        use glam::*;

        let (texture_width, texture_height) = texture_size;

        let src_rect = src_rect
            .and_then(|r| Some(r.normalized(texture_width as f32, texture_height as f32)))
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);

        let dest_rect = dest_rect.unwrap_or(Rect::new(0, 0, viewport_size.width, viewport_size.height));

        let rotation_center: [f32; 2] = rotation_center
            .unwrap_or(Point::new(
                dest_rect.x as f32 + dest_rect.w as f32 / 2.0,
                dest_rect.y as f32 + dest_rect.h as f32 / 2.0,
            ))
            .into();

        let dest_rect: [f32; 4] = dest_rect.into();

        let position = Mat4::from_translation(Vec3::new(dest_rect[0], dest_rect[1], 0.0));

        let scale = Mat4::from_scale(Vec3::new(dest_rect[2], dest_rect[3], 0.0));

        let rotation =
            Mat4::from_translation(Vec3::new(rotation_center[0], rotation_center[1], 0.0))
                * Mat4::from_rotation_z(rotation_angle)
                * Mat4::from_translation(Vec3::new(-rotation_center[0], -rotation_center[1], 0.0));

        let transform = position * rotation * scale;

        let (viewport_width, viewport_height) =
            (viewport_size.width as f32, viewport_size.height as f32);

        for vertex in UNIT_QUAD {
            let vertex = glam::Vec4::new(vertex[0], vertex[1], 0.0, 0.0);
            let position = transform * vertex;
            let x = position.x;
            let y = position.y;
            if x >= 0.0 && x <= viewport_width && y >= 0.0 && y <= viewport_height {
                return Some(Quad {
                    src_rect,
                    transform: transform.to_cols_array_2d(),
                });
            }
        }

        None
    }
}

impl QuadArrayVec {
    pub fn new() -> Self {
        let quads = QUADS_SLICE_ZEROED.to_vec();
        let next_quad = 0;
        Self {
            quads,
            next: next_quad,
        }
    }

    pub fn push(&mut self, quad: Quad) {
        // allocate more space and initialized with zero memory if needed.
        if self.next >= self.quads.len() {
            self.quads.extend_from_slice(QUADS_SLICE_ZEROED);
        }
        self.quads[self.next] = quad;
        self.next += 1;
    }

    pub fn clear(&mut self) {
        self.next = 0;
    }

    pub fn slice(&self, range: RangeFull) -> &[Quad] {
        &self.quads[range]
    }

    pub fn next(&self) -> usize {
        self.next
    }
}
