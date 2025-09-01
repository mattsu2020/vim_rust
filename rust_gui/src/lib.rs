use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[no_mangle]
pub extern "C" fn rs_gui_run() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("Vim Rust GUI")
        .build(&event_loop)
        .expect("Failed to create window");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }
    });
}
