use crate::frp::{next_id, Error, Event, InHandle, TypedInHandle};
use downcast_rs::{impl_downcast, Downcast};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// An output of a `Behaviour`.
pub struct Out<T: Event> {
    listeners: Vec<TypedInHandle<T>>,
}

impl<T: Event> Default for Out<T> {
    fn default() -> Self {
        Self { listeners: Vec::new() }
    }
}

impl<T: Event> Out<T> {
    pub fn send(&mut self, event: &T) {
        for listener in &self.listeners {
            listener.push(event);
        }
    }
}

/// Type erased version of an `Out`
pub(in crate::frp) trait GeneralOut: Downcast {
    /// Get the type of the produced event
    fn event_type_id(&self) -> TypeId;

    /// Send an event to all the connected `In`.
    /// #Panic
    /// This function may panic if the event cannect be downcasted to the type of the input.
    fn send_any(&mut self, event: &dyn Any) -> Result<(), Error>;

    /// Send an event to all the connected `In`.
    /// #Panic
    /// This function may panic if the event cannect be downcasted to the type of the input.
    fn connect_any(&mut self, handle: InHandle) -> Result<(), Error>;
}
impl_downcast!(GeneralOut);

impl<T: Event> GeneralOut for Out<T> {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn send_any(&mut self, event: &dyn Any) -> Result<(), Error> {
        let event = event.downcast_ref::<T>().ok_or(Error::UnexpectedEventType)?;
        self.send(event);
        Ok(())
    }

    fn connect_any(&mut self, handle: InHandle) -> Result<(), Error> {
        if handle.event_type_id() == TypeId::of::<T>() {
            let handle = TypedInHandle::<T>::from(handle);
            self.listeners.push(handle);
            Ok(())
        } else {
            Err(Error::UnexpectedEventType)
        }
    }
}

/// Unique id of an output set.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OutputSetId(usize);

impl OutputSetId {
    #[inline]
    pub(in crate::frp) fn new() -> Self {
        Self(next_id())
    }
}

/// The output set of a `Behaviour`.
pub trait OutputSet: 'static {
    fn id(&self) -> OutputSetId;

    /// Try to connect a new input pin to the given output pin. If their types are not matching, an error is returned.
    /// #Panic
    /// This function may panic if the index of the output is invalid.
    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error>;
}

/// Dynamic set of outputs constructed programmatically.
pub struct DynamicOutSet {
    set_id: OutputSetId,
    outputs: Vec<Box<dyn GeneralOut>>,
}

impl Default for DynamicOutSet {
    fn default() -> Self {
        Self {
            set_id: OutputSetId::new(),
            outputs: Vec::new(),
        }
    }
}

impl DynamicOutSet {
    pub fn add<T: Event>(&mut self) -> usize {
        let output = Out::<T>::default();
        let id = self.outputs.len();
        self.outputs.push(Box::new(output));
        id
    }

    pub fn get<T: Event>(&mut self, handle: TypedOutHandle<T>) -> Option<&mut Out<T>> {
        if handle.set_id() == self.set_id {
            self.outputs
                .get_mut(handle.pin_id())
                .and_then(|o| (&mut **o).downcast_mut::<Out<T>>())
        } else {
            None
        }
    }
}

impl OutputSet for DynamicOutSet {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        self.outputs[id].connect_any(in_handle)
    }
}

/// Static, compile time definition of a set of outputs.
pub struct FixedOutSet<O: Default> {
    set_id: OutputSetId,
    outputs: O,
}

impl<O: Default> Default for FixedOutSet<O> {
    fn default() -> Self {
        Self {
            set_id: OutputSetId::new(),
            outputs: O::default(),
        }
    }
}

impl<O: Default> Deref for FixedOutSet<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.outputs
    }
}

impl<O: Default> DerefMut for FixedOutSet<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.outputs
    }
}

impl OutputSet for FixedOutSet<()> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, _id: usize, _in_handle: InHandle) -> Result<(), Error> {
        panic!("Invalid id, OutputSet has no such pin");
    }
}

impl<T1: Event> OutputSet for FixedOutSet<Out<T1>> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

impl<T1: Event> OutputSet for FixedOutSet<(Out<T1>,)> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.0.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

impl<T1: Event, T2: Event> OutputSet for FixedOutSet<(Out<T1>, Out<T2>)> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.0.connect_any(in_handle),
            1 => self.outputs.1.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

impl<T1: Event, T2: Event, T3: Event> OutputSet for FixedOutSet<(Out<T1>, Out<T2>, Out<T3>)> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.0.connect_any(in_handle),
            1 => self.outputs.1.connect_any(in_handle),
            2 => self.outputs.2.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

impl<T1: Event, T2: Event, T3: Event, T4: Event> OutputSet for FixedOutSet<(Out<T1>, Out<T2>, Out<T3>, Out<T4>)> {
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.0.connect_any(in_handle),
            1 => self.outputs.1.connect_any(in_handle),
            2 => self.outputs.2.connect_any(in_handle),
            3 => self.outputs.3.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

impl<T1: Event, T2: Event, T3: Event, T4: Event, T5: Event> OutputSet
    for FixedOutSet<(Out<T1>, Out<T2>, Out<T3>, Out<T4>, Out<T5>)>
{
    fn id(&self) -> OutputSetId {
        self.set_id
    }

    fn connect(&mut self, id: usize, in_handle: InHandle) -> Result<(), Error> {
        match id {
            0 => self.outputs.0.connect_any(in_handle),
            1 => self.outputs.1.connect_any(in_handle),
            2 => self.outputs.2.connect_any(in_handle),
            3 => self.outputs.3.connect_any(in_handle),
            4 => self.outputs.4.connect_any(in_handle),
            _ => panic!("Invalid id, OutputSet has no such pin"),
        }
    }
}

/// Type erased handle to an output in an output set.
#[derive(Clone)]
pub struct OutHandle {
    set_id: OutputSetId,
    event_type: TypeId,
    pin_id: usize,
}

impl OutHandle {
    pub fn new<O: OutputSet>(output_set: &Rc<RefCell<O>>, pin_id: usize, event_type: TypeId) -> Self {
        Self {
            set_id: output_set.borrow().id(),
            event_type,
            pin_id,
        }
    }

    pub fn event_type_id(&self) -> TypeId {
        self.event_type
    }

    pub(in crate::frp) fn set_id(&self) -> OutputSetId {
        self.set_id
    }

    pub(in crate::frp) fn pin_id(&self) -> usize {
        self.pin_id
    }
}

/// Handle to an output in an output set.
#[derive(Clone)]
pub struct TypedOutHandle<T: Event> {
    handle: OutHandle,
    ph: PhantomData<T>,
}

impl<T: Event> TypedOutHandle<T> {
    pub fn new<O: OutputSet>(output_set: &Rc<RefCell<O>>, pin_id: usize) -> Self {
        Self::from(OutHandle::new(output_set, pin_id, TypeId::of::<T>()))
    }

    pub(in crate::frp) fn set_id(&self) -> OutputSetId {
        self.handle.set_id()
    }

    pub(in crate::frp) fn pin_id(&self) -> usize {
        self.handle.pin_id()
    }

    pub fn handle(&self) -> &OutHandle {
        &self.handle
    }
}

impl<T: Event> From<OutHandle> for TypedOutHandle<T> {
    /// Convert from a type erase handle.
    /// #Panic
    /// This function may panic if the types are not matching.
    fn from(handle: OutHandle) -> Self {
        assert_eq!(handle.event_type, TypeId::of::<T>());
        Self {
            handle,
            ph: PhantomData,
        }
    }
}
