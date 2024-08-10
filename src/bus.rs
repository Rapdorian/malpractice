use std::{
    fmt,
    sync::{Arc, Mutex, Weak},
};

pub struct Subscription<T> {
    queue: Mutex<Vec<T>>,
}

impl<T> Subscription<T> {
    fn push(&self, event: T) {
        self.queue.lock().unwrap().push(event);
    }

    fn next_event(&self) -> Option<T> {
        self.queue.lock().unwrap().pop()
    }
}

pub struct MessageBus<T: Copy> {
    subscribers: Mutex<Vec<Weak<Subscription<T>>>>,
}

impl<T: Copy> Default for MessageBus<T> {
    fn default() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }
}

impl<T: Copy + fmt::Debug> MessageBus<T> {
    pub fn broadcast(&self, event: T) {
        log::info!("BROADCAST: {event:?}");
        for subscriber in self.subscribers.lock().unwrap().iter() {
            if let Some(subscriber) = subscriber.upgrade() {
                subscriber.push(event);
            }
        }
    }

    pub fn subscribe(&self) -> Arc<Subscription<T>> {
        let rx = Arc::new(Subscription {
            queue: Mutex::default(),
        });
        self.subscribers.lock().unwrap().push(Arc::downgrade(&rx));
        rx
    }

    pub fn clean(&self) {
        self.subscribers
            .lock()
            .unwrap()
            .retain(|s| s.upgrade().is_some());
    }
}
