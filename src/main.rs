use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use anyhow::Result;
use app::App;


mod app;
mod instance;
mod device;
mod swapchain;
mod images;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;


    let mut destroying = false;
    let mut app = unsafe {App::Create(&window)}.unwrap();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::MainEventsCleared if !destroying => unsafe {app.render(&window)}.unwrap(),
            
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                control_flow.set_exit();
                unsafe {app.destroy()};
            },

            _ => {}
        }
    });

}