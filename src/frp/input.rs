use crate::frp::{next_id, Event};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

/// An input of a `Behaviour`.
pub trait In: 'static {
    type Event: Event;

    fn push(&mut self, event: &Self::Event) -> bool;
}

/// Type erased version of an `In`
pub(in crate::frp) trait GeneralIn {
    /// Get the type of the produced event
    fn event_type_id(&self) -> TypeId;

    /// Store a type erased event.
    /// #Panic
    /// This function may panic if the type cannot be downcasted to the type of the input.
    fn push_any(&mut self, event: &dyn Any) -> bool;
}

impl<T: In> GeneralIn for T {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn push_any(&mut self, event: &dyn Any) -> bool {
        self.push(event.downcast_ref::<T::Event>().unwrap())
    }
}

/// Unique id of an output set.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct InputSetId(usize);

impl InputSetId {
    #[inline]
    pub(in crate::frp) fn new() -> Self {
        Self(next_id())
    }
}

/// The input set of a `Behaviour`.
pub trait InputSet: 'static {
    fn id(&self) -> InputSetId;

    /// Send an event to the input with the given id.
    /// #Panic
    /// This function may panic if either the index of the input is invalid or the type
    /// cannot be downcasted to the type of the input.
    fn push(&mut self, id: usize, event: &dyn Any);

    /// Returns if event were submitted since the reset.
    fn is_dirty(&self) -> bool;

    /// Clears the dirty flag.
    fn reset_dirty(&mut self);
}

/// Dynamic set of inputs constructed programmatically.
pub struct DynamicInputSet {
    id: InputSetId,
    inputs: Vec<Box<dyn GeneralIn>>,
    dirty: bool,
}

impl Default for DynamicInputSet {
    fn default() -> Self {
        Self {
            id: InputSetId::new(),
            inputs: Vec::new(),
            dirty: false,
        }
    }
}

impl DynamicInputSet {
    pub fn add<I: In>(&mut self, input: I) -> usize {
        let id = self.inputs.len();
        self.inputs.push(Box::new(input));
        id
    }
}

impl InputSet for DynamicInputSet {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        self.dirty |= self.inputs[id].push_any(event);
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

/// Static, compile time definition of a set of inputs.
pub struct FixedInputSet<I> {
    id: InputSetId,
    inputs: I,
    dirty: bool,
}

impl<I: Default> Default for FixedInputSet<I> {
    fn default() -> Self {
        Self {
            id: InputSetId::new(),
            inputs: I::default(),
            dirty: false,
        }
    }
}

impl<I> FixedInputSet<I> {
    pub fn new(input: I) -> Self {
        Self {
            id: InputSetId::new(),
            inputs: input,
            dirty: false,
        }
    }
}

impl<I> Deref for FixedInputSet<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl<I> DerefMut for FixedInputSet<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inputs
    }
}

impl<I1: In> InputSet for FixedInputSet<I1> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.push(event.downcast_ref::<I1::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<I1: In> InputSet for FixedInputSet<(I1,)> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.0.push(event.downcast_ref::<I1::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<I1: In, I2: In> InputSet for FixedInputSet<(I1, I2)> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.0.push(event.downcast_ref::<I1::Event>().unwrap()),
            1 => self.dirty |= self.inputs.1.push(event.downcast_ref::<I2::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<I1: In, I2: In, I3: In> InputSet for FixedInputSet<(I1, I2, I3)> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.0.push(event.downcast_ref::<I1::Event>().unwrap()),
            1 => self.dirty |= self.inputs.1.push(event.downcast_ref::<I2::Event>().unwrap()),
            2 => self.dirty |= self.inputs.2.push(event.downcast_ref::<I3::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<I1: In, I2: In, I3: In, I4: In> InputSet for FixedInputSet<(I1, I2, I3, I4)> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.0.push(event.downcast_ref::<I1::Event>().unwrap()),
            1 => self.dirty |= self.inputs.1.push(event.downcast_ref::<I2::Event>().unwrap()),
            2 => self.dirty |= self.inputs.2.push(event.downcast_ref::<I3::Event>().unwrap()),
            3 => self.dirty |= self.inputs.3.push(event.downcast_ref::<I4::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<I1: In, I2: In, I3: In, I4: In, I5: In> InputSet for FixedInputSet<(I1, I2, I3, I4, I5)> {
    fn id(&self) -> InputSetId {
        self.id
    }

    fn push(&mut self, id: usize, event: &dyn Any) {
        match id {
            0 => self.dirty |= self.inputs.0.push(event.downcast_ref::<I1::Event>().unwrap()),
            1 => self.dirty |= self.inputs.1.push(event.downcast_ref::<I2::Event>().unwrap()),
            2 => self.dirty |= self.inputs.2.push(event.downcast_ref::<I3::Event>().unwrap()),
            3 => self.dirty |= self.inputs.3.push(event.downcast_ref::<I4::Event>().unwrap()),
            4 => self.dirty |= self.inputs.4.push(event.downcast_ref::<I5::Event>().unwrap()),
            _ => unreachable!(),
        }
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}

/// Type erased handle to an input in an input set.
#[derive(Clone)]
pub struct InHandle {
    input_set: Weak<RefCell<dyn InputSet>>,
    event_type: TypeId,
    pin_id: usize,
}

impl InHandle {
    pub fn new<I: InputSet>(input_set: &Rc<RefCell<I>>, pin_id: usize, event_type: TypeId) -> Self {
        let weak = Rc::downgrade(input_set);
        Self {
            input_set: weak,
            event_type,
            pin_id,
        }
    }

    pub fn event_type_id(&self) -> TypeId {
        self.event_type
    }

    pub(in crate::frp) fn push(&self, event: &dyn Any) {
        assert_eq!(event.type_id(), self.event_type);
        if let Some(input) = self.input_set.upgrade() {
            input.borrow_mut().push(self.pin_id, event);
        }
    }
}

/// Handle to an input in an input set.
pub struct TypedInHandle<T: Event> {
    handle: InHandle,
    ph: PhantomData<T>,
}

impl<T: Event> TypedInHandle<T> {
    pub fn new<I: InputSet>(input_set: &Rc<RefCell<I>>, pin_id: usize) -> Self {
        Self::from(InHandle::new(input_set, pin_id, TypeId::of::<T>()))
    }

    pub fn handle(&self) -> &InHandle {
        &self.handle
    }

    pub(in crate::frp) fn push(&self, event: &T) {
        self.handle().push(event);
    }
}

impl<T: Event> From<InHandle> for TypedInHandle<T> {
    /// Convert from a type erase handle.
    /// #Panic
    /// This function may panic if the types are not matching.
    fn from(handle: InHandle) -> Self {
        assert_eq!(handle.event_type, TypeId::of::<T>());
        Self {
            handle,
            ph: PhantomData,
        }
    }
}
