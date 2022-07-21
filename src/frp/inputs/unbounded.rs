use crate::frp::{Event, In};
use std::ops::{Deref, DerefMut};

pub struct Unbounded<T: Event>(Vec<T>);

impl<T: Event> Default for Unbounded<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: Event> Deref for Unbounded<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Event> DerefMut for Unbounded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Event> In for Unbounded<T> {
    type Event = T;

    fn push(&mut self, event: &Self::Event) -> bool {
        self.0.push(event.clone());
        true
    }
}
