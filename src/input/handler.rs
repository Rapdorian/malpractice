use std::{cmp, collections::HashMap, hash, ops};

use winit::event::WindowEvent;

use crate::input::event::Input;

use super::{action::ActionState, event::InputType};

/// This should wrap a hashmap between an action name and it's state
pub type ActionMap<A> = HashMap<A, ActionState>;

pub trait Action: hash::Hash + cmp::Eq + cmp::PartialEq + Clone {}

impl<T> Action for T where T: hash::Hash + cmp::Eq + cmp::PartialEq + Clone {}

/// Manages the input state and maps raw inputs to meaningful user-defined actions
pub struct ActionHandler<A: Action> {
    actions: ActionMap<A>,
    events: HashMap<InputType, A>,
    remap: Option<A>,
}

impl<A: Action> ActionHandler<A> {
    pub(crate) fn new() -> Self {
        Self {
            actions: Default::default(),
            events: Default::default(),
            remap: None,
        }
    }

    // Map this action to the next input by the user
    pub fn user_map(&mut self, action: A) {
        self.remap = Some(action);
    }

    /// Map a user input to an action
    pub fn map(&mut self, input: InputType, action: A) {
        self.events.insert(input, action);
    }

    /// Remove a user input from an action
    pub fn unmap(&mut self, input: InputType) {
        let _ = self.events.remove(&input);
    }

    /// Gets the mappings assigned to an action
    pub fn get_map(&self, action: A) -> Vec<InputType> {
        self.events
            .iter()
            .filter(|(_, a)| **a == action)
            .map(|(input, _)| *input)
            .collect()
    }

    /// Gets the current value of an action
    pub fn get(&self, action: &A) -> f32 {
        self.actions.get(action).map(|a| a.get()).unwrap_or(0.0)
    }

    pub(crate) fn handle_winit(&mut self, event: &WindowEvent) {
        if let WindowEvent::RedrawRequested = event {
            for action in self.actions.values_mut() {
                log::info!("::TICK::");
                action.tick();
            }
        } else {
            let input: Input = event.into();
            if input.ty == InputType::Unknown {
                return;
            }

            // Maybe remap this key
            if input.value != 0.0 {
                if let Some(action) = self.remap.take() {
                    self.map(input.ty, action);
                    return;
                }
            }

            let Some(action) = self.events.get(&input.ty) else {
                return;
            };
            log::info!("Recognized input: {:?}", input);
            let action = self
                .actions
                .entry(action.clone())
                .or_default();

            match input.ty {
                InputType::Key(_) | InputType::MouseButton(_) => {
                    if input.value > 0.5 {
                        log::info!("PRESS");
                        action.press();
                    } else {
                        log::info!("RELEASE");
                        action.release();
                    }
                }
                _ => {
                    log::info!("SET {}", input.value);
                    action.set(input.value)
                }
            }
        }
    }
}

impl<A: Action> ops::Index<A> for ActionHandler<A> {
    type Output = f32;

    fn index(&self, index: A) -> &Self::Output {
        static DEFAULT: f32 = 0.0;
        self.actions
            .get(&index)
            .map(|a| &a.value)
            .unwrap_or(&DEFAULT)
    }
}

impl<A: Action> ops::Index<&A> for ActionHandler<A> {
    type Output = f32;

    fn index(&self, index: &A) -> &Self::Output {
        static DEFAULT: f32 = 0.0;
        self.actions
            .get(index)
            .map(|a| &a.value)
            .unwrap_or(&DEFAULT)
    }
}
