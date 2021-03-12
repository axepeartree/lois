#![allow(dead_code, unused_variables, unused_mut)]

use super::{commons::AsBytes, Color, Rect};
use crate::{commons::Point, texture::Texture, ViewportSize};
use std::{collections::HashMap, collections::VecDeque, ops::Range};
use wgpu::util::DeviceExt;

pub use crate::texture::TextureDescriptor;

pub struct Pipeline2D {
    render_pipeline: wgpu::RenderPipeline,

    uniforms_layout: wgpu::BindGroupLayout,
    uniforms_bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,

    textures_layout: wgpu::BindGroupLayout,
    next_texture: TextureHandle,
    textures: HashMap<TextureHandle, Texture>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    cmd_queue: VecDeque<RenderCommand>,
    instances: Instances,
}

pub type TextureHandle = u32;

#[derive(Copy, Clone, Debug, Default)]
pub struct DrawTextureOptions {
    pub src_rect: Option<Rect>,
    pub dest_rect: Option<Rect>,
    pub center: Option<Point>,
    pub angle: f32,
    pub alpha: f32,
}

#[repr(C)]
struct Instances {
    buffer: wgpu::Buffer,
    value: Vec<Instance>,
    next: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Instance {
    src_rect: [f32; 4],
    dest_rect: [f32; 4],
    alpha: f32,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct Uniforms {
    view: [[f32; 4]; 4],
}

#[derive(Clone, Debug)]
enum RenderCommand {
    Clear {
        color: Color,
    },
    DrawTexture {
        texture: TextureHandle,
        instances: Range<u32>,
    },
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

impl Pipeline2D {
    pub fn new(device: &wgpu::Device, viewport_size: ViewportSize) -> Self {
        let textures_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let uniforms_raw = Uniforms::new(viewport_size);

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            contents: uniforms_raw.as_bytes(),
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
                    bind_group_layouts: &[&textures_layout, &uniforms_layout],
                    push_constant_ranges: &[],
                });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    buffers: &[Vertex::buffer_desc(), Instance::buffer_desc()],
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
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
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

        let textures = HashMap::with_capacity(100);

        let next_texture = 0;

        let cmd_queue = VecDeque::with_capacity(100000);

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

        let instances = Instances::new(&device);

        Self {
            render_pipeline,
            textures,
            textures_layout,
            uniforms_layout,
            uniforms_bind_group,
            uniforms_buffer,
            next_texture,
            cmd_queue,
            instances,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, viewport_size: ViewportSize) {
        queue.write_buffer(
            &self.uniforms_buffer,
            0,
            Uniforms::new(viewport_size).as_bytes(),
        );
    }

    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        descriptor: TextureDescriptor,
    ) -> Result<TextureHandle, String> {
        let texture_resource = Texture::new(device, queue, &self.textures_layout, descriptor)?;
        let texture = self.next_texture;
        self.next_texture += 1;
        let result = self.textures.insert(texture, texture_resource);
        Ok(texture)
    }

    pub fn clear(&mut self, color: Color) {
        self.cmd_queue.push_back(RenderCommand::Clear { color });
    }

    pub fn set_render_target(&mut self, texture: impl Into<Option<TextureHandle>>) {}

    pub fn draw_texture(
        &mut self,
        device: &wgpu::Device,
        texture_handle: TextureHandle,
        options: DrawTextureOptions,
        viewport_size: ViewportSize,
    ) -> Result<(), String> {
        let DrawTextureOptions {
            src_rect,
            dest_rect,
            center: _,
            angle: _,
            alpha: _,
        } = options;

        // do some naive culling
        // ignore this draw command if the quad is completely outside the viewport
        let dest_rect = dest_rect.unwrap_or(Rect {
            x: 0,
            y: 0,
            w: viewport_size.width,
            h: viewport_size.height,
        });

        // ignore this draw command if the quad is completely outside the viewport
        if !dest_rect.intersects(Rect::new(0, 0, viewport_size.width, viewport_size.height)) {
            return Ok(());
        }

        let dest_rect = dest_rect.into();

        let texture = self
            .textures
            .get(&texture_handle)
            .ok_or(String::from("Texture not found."))?;

        // src rect must be normalized (values between 0.0 and 1.0) before using it in the shader.
        let src_rect = src_rect
            .and_then(|r| Some(r.normalized(texture.width() as f32, texture.height() as f32)))
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);

        // TODO: cut the quad if only part of it is inside the viewport.

        let instance = Instance::new(src_rect, dest_rect, 1.0);

        // if there's already a draw command for this texture at the back of the queue, use it.
        if let Some(cmd) = self.cmd_queue.back_mut() {
            match cmd {
                RenderCommand::DrawTexture {
                    texture: c_texture_handle,
                    instances,
                } => {
                    if texture_handle == *c_texture_handle {
                        self.instances.push(device, instance);
                        instances.end += 1;
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        // create a new command if needed.
        self.cmd_queue.push_back(RenderCommand::DrawTexture {
            instances: self.instances.next as u32..(self.instances.next + 1) as u32,
            texture: texture_handle,
        });
        self.instances.push(device, instance);

        Ok(())
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        current_frame: &wgpu::SwapChainFrame,
    ) -> Result<(), String> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        let view = &current_frame.output.view;

        self.instances.update_buffer(queue);

        while let Some(cmd) = self.cmd_queue.pop_front() {
            match cmd {
                RenderCommand::Clear { color } => {
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Clear render pass"),
                        depth_stencil_attachment: None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                store: true,
                                load: wgpu::LoadOp::Clear(color.into()),
                            },
                        }],
                    });
                }
                RenderCommand::DrawTexture { texture, instances } => {
                    let texture = self.textures.get(&texture).expect("Texture not found.");
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Texture render pass"),
                        depth_stencil_attachment: None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: view,
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
                    render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
                    render_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, instances);
                }
            }
        }

        queue.submit(core::iter::once(encoder.finish()));

        self.instances.reset();
        Ok(())
    }
}

impl Instances {
    const ALLOC_CAPACITY: usize = 10000;

    fn new(device: &wgpu::Device) -> Self {
        let next = 0;
        let value: Vec<Instance> = vec![Default::default(); Self::ALLOC_CAPACITY];
        let buffer = {
            let slice = &value[..];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX,
                contents: slice.as_bytes(),
            })
        };

        Self {
            next: 0,
            buffer,
            value: vec![Default::default(); Self::ALLOC_CAPACITY],
        }
    }

    fn push(&mut self, device: &wgpu::Device, instance: Instance) {
        if self.next >= self.value.len() {
            self.reserve_and_recreate_buffer(device);
        }
        self.value[self.next as usize] = instance;
        self.next += 1;
    }

    fn update_buffer(&mut self, queue: &wgpu::Queue) {
        let slice = &self.value[0..self.next];
        queue.write_buffer(&self.buffer, 0, slice.as_bytes());
    }

    fn reset(&mut self) {
        self.next = 0;
    }

    fn reserve_and_recreate_buffer(&mut self, device: &wgpu::Device) {
        self.value
            .extend_from_slice(&[Default::default(); Self::ALLOC_CAPACITY]);
        self.buffer = Self::create_instances_buffer(&self.value[..], device);
    }

    fn create_instances_buffer(instances: &[Instance], device: &wgpu::Device) -> wgpu::Buffer {
        let slice = &instances[..];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::VERTEX,
            contents: slice.as_bytes(),
        })
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

impl Instance {
    fn new(src_rect: [f32; 4], dest_rect: [f32; 4], alpha: f32) -> Self {
        Self {
            src_rect,
            dest_rect,
            alpha,
        }
    }

    fn buffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
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
                    offset: core::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float,
                },
            ],
        }
    }
}

impl Uniforms {
    fn new(viewport_size: ViewportSize) -> Self {
        let left = 0.0;
        let right = viewport_size.width as f32;
        let bottom = viewport_size.height as f32;
        let top = 0.0;
        let near = 1.0;
        let far = -1.0;
        Self {
            view: Self::orthographic_rh(left, right, bottom, top, near, far),
        }
    }

    #[inline(always)]
    fn orthographic_rh(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 4]; 4] {
        let a = 2.0 / (right - left);
        let b = 2.0 / (top - bottom);
        let c = -2.0 / (far - near);
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(far + near) / (far - near);

        [
            [a, 0.0, 0.0, 0.0],
            [0.0, b, 0.0, 0.0],
            [0.0, 0.0, c, 0.0],
            [tx, ty, tz, 1.0],
        ]
    }

    #[inline(always)]
    fn rotation_around(point: Point, angle: f32) -> [[f32; 4]; 4] {
        let cos = angle.cos();
        let sin = angle.sin();
        [
            [-cos, sin, 0.0, 0.0],
            [-sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                -cos * point.x + point.x + sin * point.y,
                -sin * point.x - cos * point.y + point.y,
                0.0,
                0.0,
            ],
        ]
    }
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
        let ptr = self as *const Self;
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, size) }
    }
}

impl AsBytes for &[Instance] {
    fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Instance>() * self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, size) }
    }
}
