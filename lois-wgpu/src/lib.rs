use winit::window::Window;
use std::collections::HashMap;

use raw_window_handle::HasRawWindowHandle;
use wgpu::util::DeviceExt;

use lois::{backend::Backend, commons::{Color, ViewSize}, graphics::DrawCommand, quad::Quad, texture::{Texture, TextureFormat, TextureLoadOptions, TextureQuery, TextureUsage}};

pub struct BackendWgpu {
    viewport_size: ViewSize,
    _instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,

    _uniforms_layout: wgpu::BindGroupLayout,
    uniforms_bind_group: wgpu::BindGroup,

    texture_layout: wgpu::BindGroupLayout,
    textures: HashMap<u32, TextureWgpu>,
    next_texture: u32,

    uniforms_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: usize,
}

struct TextureWgpu {
    name: Option<String>,
    size: ViewSize,
    format: TextureFormat,
    usage: TextureUsage,

    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Vertex(f32, f32);

#[repr(C)]
#[derive(Clone, Debug)]
struct Uniforms {
    view: [[f32; 4]; 4],
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex(0.0, 0.0),
    Vertex(1.0, 0.0),
    Vertex(1.0, 1.0),
    Vertex(0.0, 1.0),
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

impl Backend for BackendWgpu {
    fn present(&mut self, commands: &[DrawCommand], quads: &[Quad]) {
        let current_frame = match self.swap_chain.get_current_frame() {
            Ok(current_frame) => current_frame,
            Err(wgpu::SwapChainError::OutOfMemory) => {
                self.resize_viewport(self.viewport_size);
                return
            }
            _ => return
        };

        let view = &current_frame.output.view;

        if self.instance_buffer_capacity < quads.len() || self.instance_buffer.is_none() {
            self.instance_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                contents: quads.as_bytes(),
            }));
            self.instance_buffer_capacity = quads.len();
        } else if let Some(instance_buffer) = self.instance_buffer.as_ref() {
            self.queue.write_buffer(instance_buffer, 0, quads.as_bytes());
        }

        let instance_buffer = self.instance_buffer.as_ref().unwrap();

        for command in commands {
            match command {
                DrawCommand::DrawTextureBatch(command) => {
                    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    });

                    let texture = self.textures.get(&command.texture.id()).expect("Texture not found while presenting.");

                    let (target, size) = if let Some(target) = command.target {
                        let texture = self.textures.get(&target.id()).expect("Target not found while presenting.");
                        (&texture.view, texture.size)
                    } else {
                        (view, self.viewport_size)
                    };

                    self.queue.write_buffer(&self.uniforms_buffer, 0, Uniforms::new(size).as_bytes());

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Texture render pass"),
                            depth_stencil_attachment: None,
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: target,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    store: true,
                                    load: wgpu::LoadOp::Load,
                                },
                            }],
                        });
                        render_pass.set_pipeline(&self.render_pipeline);
                        render_pass.set_bind_group(0, &texture.bind_group, &[]);
                        render_pass.set_bind_group(1, &self.uniforms_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass
                            .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, command.range.start as u32..command.range.end as u32);
                    }
                    self.queue.submit(core::iter::once(encoder.finish()));
                }
                DrawCommand::Clear(command) => {
                    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    });

                    let target = if let Some(target) = command.target {
                        let texture = self.textures.get(&target.id()).expect("Target not found while presenting.");
                        &texture.view
                    } else {
                        view
                    };
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Clear render pass"),
                        depth_stencil_attachment: None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                store: true,
                                load: wgpu::LoadOp::Clear(color_to_wgpu_color(command.color)),
                            },
                        }],
                    });
                    self.queue.submit(core::iter::once(encoder.finish()));
                }
            }
        }
    }

    fn load_texture(
        &mut self,
        options: TextureLoadOptions,
    ) -> Result<Texture, String> {
        let texture_resource = TextureWgpu::new(&self.device, &self.queue, &self.texture_layout, options)?;
        let texture = self.next_texture;
        self.next_texture += 1;
        self.textures.insert(texture, texture_resource);
        Ok(Texture::new(texture))
    }

    fn unload_texture(&mut self, texture: Texture) {
        self.textures.remove(&texture.id());
    }

    fn query_texture(
        &self,
        texture: Texture,
    ) -> Option<TextureQuery> {
        let texture = self.textures.get(&texture.id())?;
        Some(TextureQuery {
            name: texture.name.as_ref().map(|s| s.as_str()),
            format: texture.format,
            usage: texture.usage,
            size: texture.size,
        })
    }

    fn resize_viewport(&mut self, new_size: ViewSize) {
        self.viewport_size = new_size;
        self.swap_chain = create_swap_chain(
            &self.device,
            &self.surface,
            self.viewport_size.width,
            self.viewport_size.height,
        );
        self.queue.write_buffer(&self.uniforms_buffer, 0, Uniforms::new(new_size).as_bytes());
    }

    fn viewport(&self) -> ViewSize {
        self.viewport_size
    }
}

impl BackendWgpu {
    pub async unsafe fn new(
        window: &Window,
        viewport_size: ViewSize,
    ) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = instance.create_surface(window);

        let (device, queue) = {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    power_preference: wgpu::PowerPreference::HighPerformance,
                })
                .await
                .ok_or(String::from("Unable to request a suitable wgpu::Adapter."))?;

            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("Device"),
                        features: wgpu::Features::empty(),
                        limits: Default::default(),
                    },
                    None,
                )
                .await
                .map_err(|err| err.to_string())?
        };

        let swap_chain = create_swap_chain(&device, &surface, viewport_size.width, viewport_size.height);

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Textures Bind Group Layout Descriptor"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                },
            ],
        });

        let uniforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                ty: wgpu::BindingType::Buffer {
                    min_binding_size: None,
                    has_dynamic_offset: false,
                    ty: wgpu::BufferBindingType::Uniform,
                },
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStage::VERTEX,
            }],
        });

        let uniforms = Uniforms::new(viewport_size);

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            contents: uniforms.as_bytes(),
        });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms Bind Group"),
            layout: &uniforms_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline = {
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_layout, &uniforms_layout],
                    push_constant_ranges: &[],
                });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    buffers: &[Vertex::buffer_desc(), quads_buffer_desc()],
                    module: &device.create_shader_module(&wgpu::include_spirv!(
                        "../shaders/out/shader.vert.spv"
                    )),
                    entry_point: "main",
                },
                fragment: Some(wgpu::FragmentState {
                    module: &device.create_shader_module(&wgpu::include_spirv!(
                        "../shaders/out/shader.frag.spv"
                    )),
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        color_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha_blend: wgpu::BlendState {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Front,
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    strip_index_format: None,
                },
            })
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsage::VERTEX,
            contents: QUAD_VERTICES.as_bytes(),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            usage: wgpu::BufferUsage::INDEX,
            contents: QUAD_INDICES.as_bytes(),
        });

        Ok(Self {
            _instance: instance,
            device,
            queue,
            surface,
            swap_chain,
            viewport_size,
            index_buffer,
            instance_buffer: None,
            instance_buffer_capacity: 0,
            next_texture: 0,
            render_pipeline,
            texture_layout,
            textures: HashMap::with_capacity(100),
            uniforms_buffer,
            uniforms_bind_group,
            _uniforms_layout: uniforms_layout,
            vertex_buffer,
        })
    }
}

impl TextureWgpu {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        options: TextureLoadOptions,
    ) -> Result<Self, String> {
        let TextureLoadOptions {
            name,
            data,
            usage,
            format,
            size,
        } = options;

        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: name,
            dimension: wgpu::TextureDimension::D2,
            format: match format {
                TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
                _ => panic!("Texture format not supported.")
            },
            usage: match usage {
                TextureUsage::Default => wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
                TextureUsage::RenderTarget => wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            },
            mip_level_count: 1,
            sample_count: 1,
            size: texture_size,
        });

        let view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
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
                    bytes_per_row: 4 * size.width, // FORMAT!
                    rows_per_image: size.height,
                },
                texture_size,
            );
        }

        let name = format!("{} Texture Bind Group", name.unwrap_or("Untitled"));

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
            name: Some(name),
            _texture: texture,
            format,
            usage,
            view,
            _sampler: sampler,
            bind_group,
            size,
        })
    }
}

impl Uniforms {
    fn new(size: ViewSize) -> Self {
        let left = 0.0;
        let right = size.width as f32;
        let bottom = size.height as f32;
        let top = 0.0;
        let near = 1.0;
        let far = -1.0;
        Self {
            view: glam::Mat4::orthographic_rh(left, right, bottom, top, near, far).to_cols_array_2d()
        }
    }
}

impl Vertex {
    fn buffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float2,
            }],
        }
    }
}

trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for Uniforms {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }
}

impl AsBytes for &[Vertex] {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Vertex>() * self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, size) }
    }
}

impl AsBytes for &[u16] {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<u16>() * self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, size) }
    }
}

impl AsBytes for &[Quad] {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Quad>() * self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, size) }
    }
}

fn quads_buffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: core::mem::size_of::<Quad>() as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Instance,
        attributes: &[
            // transform
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: 2 * core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: 3 * core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Float4,
            },
            // src_rect
            wgpu::VertexAttribute {
                offset: 4 * core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 5,
                format: wgpu::VertexFormat::Float4,
            },
        ],
    }
}

fn color_to_wgpu_color(color: Color) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64 / u8::MAX as f64,
        g: color.g as f64 / u8::MAX as f64,
        b: color.b as f64 / u8::MAX as f64,
        a: color.a as f64 / u8::MAX as f64,
    }
}

fn create_swap_chain(
    device: &wgpu::Device,
    surface: &wgpu::Surface,
    width: u32,
    height: u32,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            present_mode: wgpu::PresentMode::Fifo,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
        },
    )
}
