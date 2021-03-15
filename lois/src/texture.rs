#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Texture(u32);

#[derive(Copy, Clone, Debug)]
pub struct TextureQuery<'a> {
    pub name: Option<&'a str>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TextureLoadOptions<'a> {
    pub name: Option<&'a str>,
    pub data: Option<&'a [u8]>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    Bgra8UnormSrgb,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureUsage {
    Default,
    RenderTarget,
}

impl Texture {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Default for TextureFormat {
    fn default() -> Self {
        Self::Bgra8UnormSrgb
    }
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self::Default
    }
}
