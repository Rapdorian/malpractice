use std::{cmp, ops};
use super::TimeStamp;

/// Store render state that needs to be tracked between frames
pub struct RenderState<Id, Data> {
    timestamp: TimeStamp,
    items: Vec<(Id, Data)>,
}

impl<Id, Data> RenderState<Id, Data>
where
    Id: cmp::Ord + Clone,
{

    pub(crate) fn new(timestep: f32) -> Self {
        Self {
            timestamp: TimeStamp::new(timestep as f64),
            items: Vec::new(),
        }
    }

    pub(crate) fn set_timestamp(&mut self, time: f32) {
        self.timestamp = TimeStamp::new(time as f64);
    }

    /// Get when this data was generated
    pub fn timestamp(&self) -> f64 {
        *self.timestamp
    }

    /// Move the internal timestamp forward by `dt`
    pub fn tick(&mut self, dt: f32) {
        self.timestamp.tick(dt)
    }

    /// Insert a state object
    ///
    /// Will replace any existing state with the same id
    pub fn store(&mut self, id: Id, data: Data) {
        match self.items.binary_search_by_key(&id, |item| item.0.clone()) {
            Ok(idx) => self.items[idx] = (id, data),
            Err(idx) => self.items.insert(idx, (id, data)),
        }
    }

    pub fn get(&self, id: &Id) -> Option<&Data> {
        match self.items.binary_search_by_key(id, |item| item.0.clone()) {
            Ok(idx) => Some(&self.items[idx].1),
            Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, id: &Id) -> Option<&mut Data> {
        match self.items.binary_search_by_key(id, |item| item.0.clone()) {
            Ok(idx) => Some(&mut self.items[idx].1),
            Err(_) => None,
        }
    }
}

impl<Id, Data> ops::Index<&Id> for RenderState<Id, Data>
where
    Id: cmp::Ord + Clone,
{
    type Output = Data;

    fn index(&self, index: &Id) -> &Self::Output {
        self.get(index).expect("Failed to find render state")
    }
}

impl<Id, Data> ops::IndexMut<&Id> for RenderState<Id, Data> where Id: cmp::Ord + Clone,
{
    fn index_mut(&mut self, index: &Id) -> &mut Self::Output {
        self.get_mut(index).expect("Failed to find render state")
    }

}

impl<Id, Data> ops::Index<Id> for RenderState<Id, Data>
where
    Id: cmp::Ord + Clone,
{
    type Output = Data;

    fn index(&self, index: Id) -> &Self::Output {
        self.get(&index).expect("Failed to find render state")
    }
}

impl<Id, Data> ops::IndexMut<Id> for RenderState<Id, Data> where Id: cmp::Ord + Clone,
{
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.get_mut(&index).expect("Failed to find render state")
    }

}
