use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

use crate::render_engine::RenderEngine;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    render_engine: Option<RenderEngine>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(WindowAttributes::default()) {
            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());

            let (width, height) = window_handle.inner_size().into();

            let renderer = pollster::block_on(async move {
                RenderEngine::new(window_handle.clone(), width, height).await
            });

            self.render_engine = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let (Some(window), Some(render_engine)) =
            (self.window.as_ref(), self.render_engine.as_mut())
        else {
            return;
        };
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                // Exit by pressing the escape key
                if matches!(key_code, winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                render_engine.resize(width, height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => render_engine.render_frame(),
            _ => (),
        }
        window.request_redraw();
    }
}
