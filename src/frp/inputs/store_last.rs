use crate::frp::{Event, In};
pub struct StoreLast<T: Event>(Option<T>);

impl<T: Event> Default for StoreLast<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Event> StoreLast<T> {
    pub fn new(event: T) -> Self {
        Self(Some(event))
    }

    pub fn try_get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }

    /// Get the stored value.
    /// #Panic
    /// This function may panic if no value is available.
    pub fn get(&self) -> &T {
        self.try_get().unwrap()
    }
}

impl<T: Event> In for StoreLast<T> {
    type Event = T;

    fn push(&mut self, event: &Self::Event) -> bool {
        self.0 = Some(event.clone());
        true
    }
}
