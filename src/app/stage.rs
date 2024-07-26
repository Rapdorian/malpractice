use crate::input::{ActionHandler, Action};

pub trait Stage<A: Action> {
    fn render(&self, interp: f32){}
    fn tick(&mut self, input: &mut ActionHandler<A>, step: f32){}
}
