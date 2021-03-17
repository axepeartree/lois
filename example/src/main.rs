use futures::executor::block_on;
use lois::{
    batch::TextureBatchOptions,
    commons::{Color, Rect, ViewSize},
    graphics::{DrawOptions, Graphics},
    texture::{Texture, TextureLoadOptions, TextureUsage},
};
use lois_wgpu::BackendWgpu;
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

type Gfx = Graphics<BackendWgpu>;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("lois example")
        .build(&event_loop)
        .unwrap();

    let backend = {
        let PhysicalSize { width, height } = window.inner_size();
        unsafe { block_on(BackendWgpu::new(&window, ViewSize::new(width, height))) }
    }
    .unwrap();

    let mut gfx = Gfx::new(backend);

    let target = gfx
        .load_texture(TextureLoadOptions {
            size: ViewSize {
                width: 100,
                height: 100,
            },
            usage: TextureUsage::RenderTarget,
            ..Default::default()
        })
        .unwrap();

    let kirby_texture_2 = load_texture(&mut gfx, "./assets/kirby.png");

    let mut angle = 0.0;
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            angle += 0.01;
            gfx.clear(Color::new(100, 200, 100, 255), None);
            gfx.clear(Color::new(200, 100, 100, 255), Some(target));

            gfx.new_batch(TextureBatchOptions::new(kirby_texture_2, Some(target)))
                .unwrap()
                .draw(DrawOptions {
                    dest_rect: Some(Rect::new(20, 20, 60, 60)),
                    rotation_angle: angle,
                    ..Default::default()
                });


            gfx.new_batch(TextureBatchOptions::new(target, None))
                .unwrap()
                .draw(Default::default());

            gfx.present();
            *control_flow = ControlFlow::Poll;
        }
        Event::WindowEvent { event, window_id } if window.id() == window_id => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => {
                gfx.resize_viewport(ViewSize::new(new_size.width, new_size.height));
                *control_flow = ControlFlow::Poll;
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                gfx.resize_viewport(ViewSize::new(new_inner_size.width, new_inner_size.height));
                *control_flow = ControlFlow::Poll;
            }
            _ => {}
        },
        _ => {}
    });
}

fn load_texture(graphics: &mut Gfx, path: &str) -> Texture {
    let img = image::io::Reader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_bgra8();

    graphics
        .load_texture(TextureLoadOptions {
            name: Some(path),
            size: ViewSize {
                width: img.width(),
                height: img.height(),
            },
            data: Some(&img),
            ..Default::default()
        })
        .unwrap()
}
