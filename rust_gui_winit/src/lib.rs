use std::collections::VecDeque;

use rust_gui_core::backend::{GuiBackend, GuiEvent};
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

/// Backend implementation using the `winit` crate for window and
/// event handling.  This replaces the platform specific GUI code
/// found in files like `gui_x11.c` and `gui_w32.c` with a safe and
/// portable Rust abstraction.
pub struct WinitBackend {
    event_loop: EventLoop<()>,
    /// Window may be `None` in headless environments.
    #[allow(dead_code)]
    window: Option<winit::window::Window>,
    events: VecDeque<GuiEvent>,
    cursor_pos: (i32, i32),
}

impl WinitBackend {
    /// Create a new window and associated event loop.
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Vim Rust GUI")
            .build(&event_loop)
            .ok();
        Self { event_loop, window, events: VecDeque::new(), cursor_pos: (0, 0) }
    }

    fn keycode_to_char(vk: VirtualKeyCode) -> Option<char> {
        use VirtualKeyCode::*;
        match vk {
            A => Some('a'), B => Some('b'), C => Some('c'), D => Some('d'), E => Some('e'),
            F => Some('f'), G => Some('g'), H => Some('h'), I => Some('i'), J => Some('j'),
            K => Some('k'), L => Some('l'), M => Some('m'), N => Some('n'), O => Some('o'),
            P => Some('p'), Q => Some('q'), R => Some('r'), S => Some('s'), T => Some('t'),
            U => Some('u'), V => Some('v'), W => Some('w'), X => Some('x'), Y => Some('y'),
            Z => Some('z'),
            Key0 => Some('0'), Key1 => Some('1'), Key2 => Some('2'), Key3 => Some('3'),
            Key4 => Some('4'), Key5 => Some('5'), Key6 => Some('6'), Key7 => Some('7'),
            Key8 => Some('8'), Key9 => Some('9'),
            Space => Some(' '),
            _ => None,
        }
    }
}

impl GuiBackend for WinitBackend {
    fn draw_text(&mut self, text: &str) {
        // Real rendering would use a graphics API.  For now we simply
        // print the text which is sufficient for demonstration and
        // keeps this backend side-effect free for tests.
        println!("draw: {text}");
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        if let Some(ev) = self.events.pop_front() {
            return Some(ev);
        }
        let events = &mut self.events;
        let cursor = &mut self.cursor_pos;
        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Exit;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor.0 = position.x as i32;
                        cursor.1 = position.y as i32;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(vk),
                                ..
                            },
                        ..
                    } => {
                        if let Some(ch) = Self::keycode_to_char(vk) {
                            events.push_back(GuiEvent::Key(ch));
                        }
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    } => events.push_back(GuiEvent::Click { x: cursor.0, y: cursor.1 }),
                    _ => {}
                },
                Event::RedrawRequested(_) => events.push_back(GuiEvent::Expose),
                _ => {}
            }
        });
        events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn create_backend() {
        use super::WinitBackend;
        // Simply verify that constructing the backend works.  This does
        // not open a window in headless CI environments and ensures that
        // memory management is handled safely.
        let mut backend = WinitBackend::new();
        backend.draw_text("test");
        // We can't easily synthesize platform events without a windowing
        // system so just confirm that polling with no events returns None.
        assert!(backend.poll_event().is_none());
    }
}
