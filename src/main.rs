use std::sync::Arc;

use winit::{self, application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};
use wgpu;

mod evolutionator;



pub struct State
{
    window: Arc<Window>,
}
impl State
{
    pub async fn new(window: Arc<Window>) -> Result<Self>
    {
        Ok(Self {window})
    }

    pub fn resize()
    {

    }

    pub fn render()
    {

    }
}



pub struct App
{
    state: Option<State>,
}

impl App
{
    pub fn new() -> Self
    {
        Self {state: None}
    } // pub fn new
} // impl App

impl ApplicationHandler<State> for App
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop)
    {   
        let mut window_attributes = Window::default_attributes();
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

    let mut app = App::new();

    el.run_app(&mut app);

}
