#[derive(Copy, Clone, Debug)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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

impl ViewportSize {
    #[inline(always)]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Into<[f32; 4]> for Rect {
    #[inline(always)]
    fn into(self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.w as f32, self.h as f32]
    }
}

impl Into<[f32; 2]> for Point {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}