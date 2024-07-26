//! This is going to be a root of a game engine I'm working on.
//!
//! First lets start with some basic windowing and a renderer.
//! we also need a decent logging framework

pub mod render;
pub mod components;
pub mod app;
pub mod input;
pub mod bus;

pub use app::*;
//pub mod window;

// Ok, so winit is annoying and doesn't allow polling events, probably for good reason.
// so in order to support that API we'll have the game engine take over the update loop of all games.
// This is not entirely bad as it more easily allows having a good timestep

// To accommodate this we'll need to pass two closures into the engines update loop.
// mod config;
// pub mod console;
// pub mod input;
// pub mod net;
// mod smol;
// pub mod utils;
// pub mod event;
pub mod world;

// use crate::config::Config;
// use crate::input::{KeyState, Keys};
// use crate::smol::{LocalExecutor, Task};
// use crate::window::{Window, WINDOW_QUEUE};
// use async_io::Timer;
// use futures::future::join_all;
// use once_cell::unsync::Lazy;
// use std::cell::Cell;
// use std::error::Error;
// use std::future::Future;
// use std::io::ErrorKind::TimedOut;
// use std::ops::Bound::Excluded;
// use std::process::exit;
// use std::thread;
// use std::time::{Duration, Instant};
// use tracing::{debug_span, error, Span};
// use winit::event::{ElementState, Event, WindowEvent};
// use winit::event_loop::{EventLoop, EventLoopWindowTarget};
// use winit::keyboard::KeyCode::Insert;
// use winit::keyboard::{KeyCode, PhysicalKey};
// 
// pub trait Scene {
//     fn update(
//         self: Box<Self>,
//         ctx: &mut EngineContext,
//     ) -> Result<Box<dyn Scene>, Box<dyn std::error::Error>>;
//     fn render(&mut self) {}
// 
//     fn span(&self) -> Span;
// }
// 
// impl<F> Scene for F
// where
//     F: FnMut(&mut EngineContext) -> Result<Option<Box<dyn Scene>>, Box<dyn std::error::Error>>
//         + 'static,
// {
//     fn update(
//         mut self: Box<Self>,
//         ctx: &mut EngineContext,
//     ) -> Result<Box<dyn Scene>, Box<dyn Error>> {
//         match (self)(ctx) {
//             Ok(None) => Ok(self),
//             Ok(Some(scene)) => Ok(scene),
//             Err(e) => Err(e),
//         }
//     }
// 
//     fn span(&self) -> Span {
//         debug_span!("Simple Scene")
//     }
// }
// 
// thread_local! { static EXECUTOR: LocalExecutor<'static> = LocalExecutor::new()}
// pub fn spawn<T: 'static>(f: impl Future<Output = T> + 'static) -> Task<T> {
//     EXECUTOR.with(|ex| ex.spawn(f))
// }
// 
// pub fn block_on<T>(future: impl Future<Output = T>) -> T {
//     EXECUTOR.with(|ex| futures::executor::block_on(ex.run(future)))
// }
// 
// pub struct EngineContext {
//     pub keys: Keys,
//     pub config: Config,
// }
// 
// impl EngineContext {
//     fn new() -> Self {
//         Self {
//             keys: Keys::default(),
//             config: Config::default(),
//         }
//     }
// }
// 
// /// main engine entrypoint
// /// call this method to start the engine
// pub fn run<A: Scene>(app: A) {
//     // TODO: improve this API surface.
//     let event_loop = EventLoop::new().unwrap();
//     let mut context = EngineContext::new();
// 
//     // un-simulated time
//     let mut t = Duration::new(0, 0);
//     let mut last_frame_time = Instant::now();
//     let mut scene: Cell<Option<Box<dyn Scene>>> = Cell::new(Some(Box::new(app)));
// 
//     // spwan a thread to redraw all windows.
//     let framerate = context.config.framerate.duration();
//     thread::spawn(move || loop {
//         Window::redraw_all();
//         thread::sleep(framerate);
//     });
// 
//     event_loop
//         .run(move |event, event_loop| {
//             // handle window creation.
//             // there will be a list of window requests in rivik::window that need to be given backings
//             for req in WINDOW_QUEUE.lock().unwrap().iter() {
//                 req.back(event_loop);
//             }
//             WINDOW_QUEUE.lock().unwrap().clear();
// 
//             if let Event::WindowEvent {
//                 window_id,
//                 event: WindowEvent::CloseRequested,
//             } = event
//             {
//                 Window::close_id(&window_id);
//             }
//             // TODO: Allow having no windows open by config
//             if Window::open_window_count() == 0 {
//                 event_loop.exit()
//             }
// 
//             match event {
//                 Event::NewEvents(_) => {}
//                 Event::WindowEvent { window_id, event } => match event {
//                     WindowEvent::ActivationTokenDone { .. } => {}
//                     WindowEvent::Resized(_) => {}
//                     WindowEvent::Moved(_) => {}
//                     WindowEvent::CloseRequested => {}
//                     WindowEvent::Destroyed => {}
//                     WindowEvent::DroppedFile(_) => {}
//                     WindowEvent::HoveredFile(_) => {}
//                     WindowEvent::HoveredFileCancelled => {}
//                     WindowEvent::Focused(_) => {}
//                     WindowEvent::KeyboardInput {
//                         device_id,
//                         event,
//                         is_synthetic,
//                     } => {
//                         let code = match event.physical_key {
//                             PhysicalKey::Code(code) => code,
//                             PhysicalKey::Unidentified(_) => KeyCode::Abort,
//                         };
//                         match (event.state, context.keys.by_code(code)) {
//                             (ElementState::Pressed, Some(key)) => key.press(),
//                             (ElementState::Released, Some(key)) => key.release(),
//                             _ => {}
//                         }
//                     }
//                     WindowEvent::ModifiersChanged(_) => {}
//                     WindowEvent::Ime(_) => {}
//                     WindowEvent::CursorMoved { .. } => {}
//                     WindowEvent::CursorEntered { .. } => {}
//                     WindowEvent::CursorLeft { .. } => {}
//                     WindowEvent::MouseWheel { .. } => {}
//                     WindowEvent::MouseInput { .. } => {}
//                     WindowEvent::TouchpadMagnify { .. } => {}
//                     WindowEvent::SmartMagnify { .. } => {}
//                     WindowEvent::TouchpadRotate { .. } => {}
//                     WindowEvent::TouchpadPressure { .. } => {}
//                     WindowEvent::AxisMotion { .. } => {}
//                     WindowEvent::Touch(_) => {}
//                     WindowEvent::ScaleFactorChanged { .. } => {}
//                     WindowEvent::ThemeChanged(_) => {}
//                     WindowEvent::Occluded(_) => {}
//                     WindowEvent::RedrawRequested => {
//                         {
//                             let s = scene.get_mut().as_mut().unwrap();
//                             let span = s.span();
//                             let _span = span.enter();
//                             s.render();
//                         }
// 
//                         t += context.config.framerate.duration();
//                         while t >= context.config.physics_step.duration() {
//                             let s = scene.take().expect("scene should exist");
//                             let span = s.span();
//                             let _span = span.enter();
//                             match s.update(&mut context) {
//                                 Ok(s) => {
//                                     scene.set(Some(s));
//                                     t -= context.config.physics_step.duration();
//                                 }
//                                 Err(e) => {
//                                     error!("{}", e);
//                                     /* TODO: Don't abort on error */
//                                     exit(0);
//                                 }
//                             }
//                         }
// 
//                         context.keys.tick();
//                     }
//                 },
//                 Event::DeviceEvent { .. } => {}
//                 Event::UserEvent(_) => {}
//                 Event::Suspended => {}
//                 Event::Resumed => {}
//                 Event::AboutToWait => {}
//                 Event::LoopExiting => {}
//                 Event::MemoryWarning => {}
//             }
//         })
//         .unwrap();
// }
