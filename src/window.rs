//! Platform specific window handling.
//! I absolutely HATE this api. It needs to be redone

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use tracing::{error, info, instrument};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{WindowBuilder, WindowId};

#[derive(Debug, Clone)]
pub(crate) struct WindowRequest {
    title: String,
}

#[derive(Debug)]
pub(crate) enum WindowInner {
    Request(WindowRequest),
    Backed(winit::window::Window),
    Closed,
}

impl WindowInner {
    pub fn request(&self) -> Option<WindowRequest> {
        match self {
            WindowInner::Request(req) => Some(req.clone()),
            WindowInner::Backed(_) => None,
            WindowInner::Closed => None,
        }
    }

    pub fn window(&self) -> Option<&winit::window::Window> {
        match self {
            WindowInner::Backed(w) => Some(w),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Window {
    inner: Arc<Mutex<WindowInner>>,
}

impl Debug for Window {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.inner.lock().as_ref() {
            Ok(inner) => inner.fmt(f),
            Err(e) => write!(f, "{}", e),
        }
    }
}

pub(crate) static WINDOW_QUEUE: Lazy<Mutex<Vec<Window>>> = Lazy::new(|| Mutex::new(vec![]));

// Need to be able to invalidate windows by id.
static WINDOW_LIST: Lazy<Mutex<HashMap<WindowId, Arc<Mutex<WindowInner>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl Window {
    /// Request a new window to be created
    pub fn new<T: Into<String>>(title: T) -> Self {
        let window = Window {
            inner: Arc::new(Mutex::new(WindowInner::Request(WindowRequest {
                title: title.into(),
            }))),
        };
        WINDOW_QUEUE.lock().unwrap().push(window.clone());
        window
    }

    pub fn is_created(&self) -> bool {
        self.with_winit(|_| {}).is_some()
    }

    pub(crate) fn with_winit<T, F: FnOnce(&winit::window::Window) -> T>(&self, f: F) -> Option<T> {
        let inner = self.inner.lock().unwrap();
        let win = inner.window();
        if let Some(win) = win {
            Some((f)(win))
        } else {
            None
        }
    }

    pub(crate) fn inner(&self) -> std::sync::MutexGuard<'_, WindowInner> {
        self.inner.lock().unwrap()
    }

    pub fn back(&self, target: &EventLoopWindowTarget<()>) {
        let request = self.inner.lock().unwrap().request().unwrap();

        info!("Initializing window: {:?}", request);

        let win = WindowBuilder::new()
            .with_title(&request.title)
            .build(target)
            .unwrap();
        WINDOW_LIST
            .lock()
            .unwrap()
            .insert(win.id(), self.inner.clone());
        // start rendering in window
        win.request_redraw();
        *self.inner.lock().unwrap() = WindowInner::Backed(win);
    }

    pub fn close(&self) {
        *self.inner.lock().unwrap() = WindowInner::Closed;
    }

    pub fn close_id(id: &WindowId) {
        *WINDOW_LIST
            .lock()
            .unwrap()
            .get_mut(id)
            .unwrap()
            .lock()
            .unwrap() = WindowInner::Closed;
    }

    pub fn open_window_count() -> usize {
        WINDOW_LIST
            .lock()
            .unwrap()
            .iter()
            .map(|(_, e)| !matches!(*e.lock().unwrap(), WindowInner::Closed))
            .filter(|e| *e)
            .count()
    }

    #[instrument]
    pub(crate) fn redraw_all() {
        for (_id, win) in WINDOW_LIST.lock().unwrap().iter() {
            let window = win.lock().unwrap();
            if let Some(window) = window.window() {
                window.request_redraw();
            }
        }
    }

    pub(crate) fn redraw_window_id(id: WindowId) {
        WINDOW_LIST
            .lock()
            .unwrap()
            .get(&id)
            .unwrap()
            .lock()
            .unwrap()
            .window()
            .unwrap()
            .request_redraw();
    }
}
