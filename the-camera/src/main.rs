use app::App;
use winit::event_loop::EventLoop;
mod app;
mod mesh;
mod render_engine;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll); // Proceed with next loop iteration right after prior finishes

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
