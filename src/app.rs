//! Application lifecycle management.

// ok, brainstorm time.
// we need to store a decent amount of per stage/window data at the engine level.
// so we need to support that cleaner.

// let rivik = rivik::new()
// rivik.map_input(...);
// rivik.map_input(...);
// rivik.run(|r| r.new_stage(MainMenu::new());

mod stage;
mod stage_builder;

pub use stage::*;

use crate::app::stage_builder::StageBuilder;
use crate::render::Surface;
use crate::{
    input::{Action, ActionHandler},
    render::{RenderState, TimeStamp},
};
use std::ops::{Deref, DerefMut};
use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    time::Instant,
};
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowAttributes;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};



pub struct ActiveRivik<'a, A: Action> {
    rivik: &'a mut Rivik<A>,
    event_loop: &'a ActiveEventLoop,
}

impl<'a, A: Action> Deref for ActiveRivik<'a, A> {
    type Target = Rivik<A>;

    fn deref(&self) -> &Self::Target {
        &self.rivik
    }
}

impl<'a, A: Action> DerefMut for ActiveRivik<'a, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.rivik
    }
}

impl<'a, A: Action> ActiveRivik<'a, A> {
    pub fn open(&mut self, window: WindowAttributes) -> StageBuilder<A> {
        let window = Arc::new(self.event_loop.create_window(window).unwrap());
        StageBuilder {
            window,
            rivik: self.rivik,
            surface: None,
        }
    }
}

pub struct Rivik<A: Action> {
    init: Option<Box<dyn FnOnce(&mut ActiveRivik<A>)>>,
    stages: HashMap<WindowId, EngineStage<A>>,
    input: ActionHandler<A>,
    sim_time: TimeStamp,
    timestep: f32,
    prev_frametime: Option<Instant>,
    render_state: RenderState<u32, ()>,
}

impl<A: Action> Rivik<A> {
    pub fn new() -> Self {
        Self {
            init: None,
            stages: Default::default(),
            input: ActionHandler::new(),
            sim_time: TimeStamp::default(),
            timestep: 1.0 / 20.0,
            prev_frametime: None,
            render_state: RenderState::new(0.0),
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn run(stage: impl FnOnce(&mut ActiveRivik<A>) + 'static) {
        let mut rivik = Rivik::new();
        let event_loop = EventLoop::new().unwrap();
        rivik.init = Some(Box::new(stage));
        event_loop.run_app(&mut rivik).unwrap()
    }

    #[cfg(target_os = "android")]
    pub fn run(
        stage: impl FnOnce(&mut ActiveRivik<A>) + 'static,
        app: winit::platform::android::activity::AndroidApp,
    ) {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        use crate::assets::android::ANDROID_APP;

        ANDROID_APP.set(app.clone()).unwrap();
        let mut rivik = Rivik::new();
        let event_loop = EventLoopBuilder::default()
            .with_android_app(app)
            .build()
            .unwrap();
        rivik.init = Some(Box::new(stage));
        event_loop.run_app(&mut rivik).unwrap()
    }

    pub fn input(&mut self) -> &mut ActionHandler<A> {
        &mut self.input
    }

    fn active<'a>(&'a mut self, event_loop: &'a ActiveEventLoop) -> ActiveRivik<'a, A> {
        ActiveRivik {
            rivik: self,
            event_loop,
        }
    }
}

impl<A: Action> ApplicationHandler for Rivik<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Starting Rivik Engine");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        if let Some(init) = self.init.take() {
            (init)(&mut self.active(event_loop));
        }
        for (_id, stage) in &mut self.stages {
            stage.resume();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        for (_id, stage) in &mut self.stages {
            stage.suspend();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // quick overview of how the gameloop should operate.
        // Time is generated by the game simulation in tick()
        // it pushes the current timestamp forward.
        // When we call render() it will check the dt since the last frame
        // and move the rendertimestamp forward by that much. (should always be behind the tick()
        // timestamp) We then feed that timestamp into the renderer as well as a way to record
        // render state (positions mostly) and the previous frame's render state. The renderer
        // SHOULD then interpolate between previous and current states using the timestamps.

        // feed event to egui
        let mut app_bench = Some(crate::bench::start("app_mgmt"));
        self.stages
            .get_mut(&window_id)
            .unwrap()
            .raw_window_event(&event);

        match &event {
            WindowEvent::CloseRequested => {
                let _ = self.stages.remove(&window_id).unwrap();
                if self.stages.len() == 0 {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                self.stages.get_mut(&window_id).unwrap().resize();
            }
            WindowEvent::RedrawRequested => {
                let time = Instant::now();
                let p_time = self.prev_frametime.unwrap_or_else(|| Instant::now());
                let dt = (time - p_time).as_secs_f32();
                self.prev_frametime = Some(time);

                //log::error!("====== dt: {dt}");
                //log::error!(" - sim_time: \t{}", *self.sim_time);
                //log::error!(" - vis_time: \t{}", self.render_state.timestamp());

                let catchup = (*self.sim_time - self.render_state.timestamp()) as f32;
                //log::error!(" - catchup:  \t{}", catchup);

                if catchup < self.timestep / 2.0 {
                    app_bench.take();
                    self.stages
                        .get_mut(&window_id)
                        .unwrap()
                        .tick(&mut self.input, self.timestep);
                    app_bench = Some(crate::bench::start("app_mgmt"));
                    self.sim_time.tick(self.timestep);
                }

                // assume a range vis_time..sim_time mapped to 0..1
                // we need to figure out where vis_time+dt is
                let s = dt as f64 / (*self.sim_time - self.render_state.timestamp());

                app_bench.take();
                self.stages
                    .get_mut(&window_id)
                    .unwrap()
                    .render(s as f32, &mut self.input);
                app_bench = Some(crate::bench::start("app_mgmt"));
                self.render_state.tick(dt);
            }
            _ => {}
        }
        self.input.handle_winit(&event);
    }
}
