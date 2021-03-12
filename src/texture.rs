#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: (u32, u32),
    pub bind_group: wgpu::BindGroup,
}

pub struct TextureDescriptor<'a> {
    pub name: Option<&'a str>,
    pub data: Option<&'a [u8]>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        descriptor: TextureDescriptor,
    ) -> Result<Self, String> {
        let TextureDescriptor {
            name,
            data,
            width,
            height,
        } = descriptor;

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: name,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            size: texture_size,
        });

        let view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        if let Some(data) = data {
            queue.write_texture(
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * width,
                    rows_per_image: height,
                },
                texture_size,
            );
        }

        let name = format!("{} Bind Group", name.unwrap_or("Untitled"));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name.as_str()),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
            size: (width, height),
        })
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.size.0
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.size.1
    }
}
