# lois

Small 2D renderer prototype based on WGPU.

## Usage

```rust
use lois::*;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Fucking MacOS")
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();

    let mut gfx = GraphicsState::new(&window, size.width, size.height).unwrap();

    let texture = load_texture("my_texture.png", &mut gfx).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {}
        Event::MainEventsCleared => {
            gfx.clear(Color([100, 149, 237, 255]));
            gfx.draw_texture(texture, None, None).unwrap();
            gfx.render().unwrap();
        }
        _ => {}
    });
}

fn load_texture(path: &str, gfx: &mut GraphicsState) -> Result<TextureHandle, String> {
    let img = image::io::Reader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    gfx.load_texture(TextureDescriptor {
        name: Some("assets/happy_tree.png"),
        width: img.width(),
        height: img.height(),
        data: &img,
    })
}
```

## TODOs

1. Render targets
2. Texture rotation
3. Culling
4. Stencils
