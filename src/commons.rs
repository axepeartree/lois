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
    #[inline(always)]
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    #[inline(always)]
    pub(crate) fn normalized(&self, width: f32, height: f32) -> [f32; 4] {
        let x = self.x as f32 / width;
        let y = self.y as f32 / height;
        let w = self.w as f32 / width;
        let h = self.h as f32 / height;
        [x, y, w, h]
    }
}

impl Into<wgpu::Color> for Color {
    #[inline(always)]
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
    #[inline(always)]
    fn into(self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.w as f32, self.h as f32]
    }
}