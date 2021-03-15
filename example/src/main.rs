use futures::executor::block_on;
use lois::{batch::TextureBatchOptions, commons::{Color, ViewportSize}, graphics::Graphics, texture::TextureLoadOptions};
use lois_wgpu::BackendWgpu;
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("lois example")
        .build(&event_loop)
        .unwrap();


    let backend = {
        let size = window.inner_size();
        let (width, height) = (size.width, size.height);
        unsafe { block_on(BackendWgpu::new(&window, ViewportSize::new(width, height))) }
    }.unwrap();

    let mut graphics = Graphics::new(backend);

    let texture = {
        let img = image::io::Reader::open("assets/kirby.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();

        graphics.load_texture(TextureLoadOptions {
            name: Some("kirby"),
            width: img.width(),
            height: img.height(),
            data: Some(&img),
            ..Default::default()
        }).unwrap()
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            graphics.clear(None, Color {
                r: 100,
                g: 200,
                b: 100,
                a: 255,
            });
            let mut batch = graphics.new_batch(TextureBatchOptions { texture, target: None }).unwrap();
            batch.draw(Default::default());
            graphics.present();
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, window_id } if window.id() == window_id => {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        }
        _ => {}
    });
}
