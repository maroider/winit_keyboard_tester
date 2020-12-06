use simple_logger::SimpleLogger;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic,
                    ..
                },
            ..
        } = &event
        {
            if !event.repeat {
                println!(
                    "{} {:?} {:?} {} {} {:?}",
                    is_synthetic,
                    event.scancode,
                    event.location,
                    event.physical_key,
                    event.logical_key,
                    event.state,
                );
            }
        }

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
