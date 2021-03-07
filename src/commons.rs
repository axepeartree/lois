#[derive(Copy, Clone, Debug)]
pub struct Color(pub [u8; 4]);

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.0[0] as f64 / 255.0,
            g: self.0[1] as f64 / 255.0,
            b: self.0[2] as f64 / 255.0,
            a: self.0[3] as f64 / 255.0,
        }
    }
}

impl Into<[f32; 4]> for Rect {
    fn into(self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.w as f32, self.h as f32]
    }
}