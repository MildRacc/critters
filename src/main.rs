use std::sync::Arc;

use winit::{self, application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};
use wgpu;

mod evolutionator;



#[derive(Default)]
struct App
{

    window: Arc<Window>,

}

impl ApplicationHandler for App
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop)
    {
       self.window = event_loop.create_window(Window::default_attributes()).unwrap();
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        )
    {
        match event {
            WindowEvent::CloseRequested => {event_loop.exit();},
            WindowEvent::RedrawRequested => {
                
                // Draw here
                
                self.window.request_redraw();

            },

            _ => (),
        } // Match Event
    } // fn window_event
} // impl Application for App


fn main() {
    println!("Hello, world!");

    let el = EventLoop::new().unwrap();

    el.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    el.run_app(&mut app);

}
