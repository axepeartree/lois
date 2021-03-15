use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("lois example")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
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
