use std::num::NonZeroU32;

use anyhow::Result;
use glow::HasContext;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::{Display, DisplayApiPreference},
    prelude::{GlDisplay, NotCurrentGlContext as _, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

struct App {
    window: Option<Window>,
    surface: Option<Surface<WindowSurface>>,
    context: Option<PossiblyCurrentContext>,
    gl: Option<glow::Context>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize::new(800, 600))
                    .with_visible(true),
            )
            .unwrap();
        let window_handle = window.window_handle().unwrap();

        let display_handle = window.display_handle().unwrap();
        #[cfg(target_os = "windows")]
        let api_preference = DisplayApiPreference::Wgl(Some(window_handle.as_raw()));
        #[cfg(target_os = "macos")]
        let api_preference = DisplayApiPreference::Cgl;
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let api_preference = DisplayApiPreference::Egl;
        let display = unsafe { Display::new(display_handle.as_raw(), api_preference).unwrap() };
        let template = ConfigTemplateBuilder::default().build();
        let config = unsafe { display.find_configs(template).unwrap().next().unwrap() };

        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::default().build(
            window_handle.as_raw(),
            NonZeroU32::new(window.inner_size().width).unwrap(),
            NonZeroU32::new(window.inner_size().height).unwrap(),
        );
        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attrs)
                .unwrap()
        };
        let context_attrs = ContextAttributesBuilder::default().build(Some(window_handle.as_raw()));
        let context = unsafe { display.create_context(&config, &context_attrs).unwrap() };
        let context = context.make_current(&surface).unwrap();

        let gl =
            unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };

        self.window = Some(window);
        self.surface = Some(surface);
        self.context = Some(context);
        self.gl = Some(gl);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                let gl = self.gl.as_ref().unwrap();
                let context = self.context.as_ref().unwrap();
                let surface = self.surface.as_ref().unwrap();

                window.pre_present_notify();
                context.make_current(surface).unwrap();
                unsafe {
                    gl.clear_color(1.0, 0.0, 0.0, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                    surface.swap_buffers(context).unwrap();
                }
            }
            _ => (),
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        window: None,
        surface: None,
        context: None,
        gl: None,
    };
    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(&mut app)?;

    Ok(())
}
