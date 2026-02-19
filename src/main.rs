use std::sync::Arc;

use winit::{self, application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};
use wgpu;

mod evolutionator;


#[derive(Clone)]
pub struct State
{
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
}
impl State
{
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self>
    {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor{backends: wgpu::Backends::PRIMARY, ..Default::default()});

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label: None,
            required_features: wgpu::Features::all(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::defaults(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        }

        

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window
        })
    }

    pub fn resize()
    {

    }

    pub fn render()
    {

    }

    pub fn handle_keys()
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
                if let Some(state) = self.state.clone()
                {
                    state.window.request_redraw();
                }
            },

            _ => (),
        } // Match Event
    } // fn window_event
} // impl Application for App


fn main() {
    println!("Hello, world!");

    let el = EventLoop::with_user_event().build().unwrap();

    el.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    el.run_app(&mut app);

}
