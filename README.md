# lois

Small 2D renderer prototype based on WGPU. Automatically does texture batching.

## Usage

```rust
use lois::*;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use futures::executor::block_on;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Application")
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();

    let mut gfx = block_on(GraphicsState::new(&window, size.width, size.height)).unwrap();

    // load texture as &[u8] and  use the GraphicsState variable to load it into the GPU.
    let texture = {
        let img = image::io::Reader::open("my_texture.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

        gfx.load_texture(TextureDescriptor {
            name: Some("my_texture"),
            width: img.width(),
            height: img.height(),
            data: &img,
        })
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            gfx.clear(Color([100, 149, 237, 255]));
            gfx.draw_texture(texture, None, None).unwrap();
            // draw calls are only presented once render is called.
            gfx.render().unwrap();
        }
        _ => {}
    });
}
```

## TODOs

1. Render into texture
2. Rotate texture
3. CPU Culling
4. Stencils
5. Reallocate buffers once max instance count is reached (current behaviour is panicking).