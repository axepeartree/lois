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

impl Quad {
    const fn empty() -> Self {
        Self {
            transform: [[0.0,0.0,0.0,0.0]; 4],
            src_rect: [0.0,0.0,0.0,0.0]
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
        todo!("Quad creation not implemented!")
    }
}

impl QuadArrayVec {
    pub fn new() -> Self {
        let quads = QUADS_SLICE_ZEROED.to_vec();
        let next_quad = 0;
        Self { quads, next: next_quad }
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
