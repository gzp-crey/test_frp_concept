use crate::frp::{
    Behaviour, DynamicOutSet, Error, InHandle, InputSet, InputSetId, IntoBehaviourNode, OutHandle, OutputSet,
    OutputSetId, TypedInHandle, TypedOutHandle,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};

use super::GeneralBehaviourNode;

pub trait Event: 'static + Clone {}
impl<T> Event for T where T: 'static + Clone {}

/// Store an FRP graph.
pub struct System {
    /// input of the system that triggers the execution of the graph
    system_inputs: Rc<RefCell<DynamicOutSet>>,
    /// output of the system that can trigger the clients of the graph
    //outputs: DynamicOutputSet,
    /// References to all the `InputSet`s in this system
    input_set_references: HashMap<InputSetId, Weak<RefCell<dyn InputSet>>>,
    /// References to all the `OutputSet`s in this system
    output_set_references: HashMap<OutputSetId, Weak<RefCell<dyn OutputSet>>>,
    nodes: Vec<Box<dyn GeneralBehaviourNode>>,
}

impl Default for System {
    fn default() -> Self {
        let system_inputs = Rc::new(RefCell::new(DynamicOutSet::default()));

        let input_set_references: HashMap<InputSetId, Weak<RefCell<dyn InputSet>>> = HashMap::new();
        let output_set_references = {
            let mut output_set_references: HashMap<OutputSetId, Weak<RefCell<dyn OutputSet>>> = HashMap::new();
            let set_id = system_inputs.borrow().id();
            let weak = Rc::downgrade(&system_inputs);
            output_set_references.insert(set_id, weak);
            output_set_references
        };

        Self {
            system_inputs,
            input_set_references,
            output_set_references,
            nodes: Vec::new(),
        }
    }
}

impl System {
    /// Create a new input for the system.
    pub fn create_input<T: Event>(&mut self) -> TypedOutHandle<T> {
        let pin_id = {
            let inputs = &mut *self.system_inputs.borrow_mut();
            inputs.add::<T>()
        };
        TypedOutHandle::new(&self.system_inputs, pin_id)
    }

    /// Add a new behaviour to the system.
    pub fn add_behaviour<B: IntoBehaviourNode>(
        &mut self,
        behaviour: B,
    ) -> Result<<B::Behaviour as Behaviour>::PinLayout, Error> {
        let behaviour = behaviour.into_behaviour_node()?;
        self.add_input_set_reference(&behaviour.input_set);
        self.add_output_set_reference(&behaviour.output_set);
        let pin_layout = behaviour.get_pins();
        self.nodes.push(Box::new(behaviour));
        Ok(pin_layout)
    }

    /// Try to connect the output and input, see `connect_any`
    pub fn connect<T: Event>(&mut self, pin_out: &TypedOutHandle<T>, pin_in: &TypedInHandle<T>) -> Result<(), Error> {
        self.connect_any(pin_out.handle(), pin_in.handle())
    }

    /// Try to connect the output and input.
    /// The operation fails if either the type of the input and output are not matching ot the connection would create a cycle in the graph.
    pub fn connect_any(&mut self, pin_out: &OutHandle, pin_in: &InHandle) -> Result<(), Error> {
        if pin_out.event_type_id() != pin_in.event_type_id() {
            Err(Error::IncompatiblePinTypes)
        } else {
            // todo: create topolgy ordering with cycle detection
            // todo2: make update inceremntal, see: https://www.researchgate.net/publication/47841865_Maintaining_Longest_Paths_Incrementally            

            let out_set = Arc::new(
                self.output_set_references
                    .get(&pin_out.set_id())
                    .ok_or(Error::OutputNotFound)?,
            )
            .upgrade()
            .unwrap();
            out_set.borrow_mut().connect(pin_out.pin_id(), pin_in.clone())?;
            Ok(())
        }
    }

    /// Send an event to an input of the system and run the graph to completion.
    /// #Panic
    /// This function may panic if the input handle is not an input of the system.
    pub fn run_on<T: Event>(&mut self, input: TypedOutHandle<T>, event: &T) -> Result<(), Error> {
        {
            let inputs = &mut *self.system_inputs.borrow_mut();
            let input = inputs.get(input).ok_or(Error::InputNotFound)?;
            input.send(event);
        }
        self.run();
        Ok(())
    }

    fn add_input_set_reference<I: InputSet>(&mut self, input_set: &Rc<RefCell<I>>) {
        let set_id = input_set.borrow().id();
        let weak = Rc::downgrade(input_set);
        self.input_set_references.insert(set_id, weak);
    }

    fn add_output_set_reference<O: OutputSet>(&mut self, output_set: &Rc<RefCell<O>>) {
        let set_id = output_set.borrow().id();
        let weak = Rc::downgrade(output_set);
        self.output_set_references.insert(set_id, weak);
    }

    fn run(&mut self) {
        for node in &mut self.nodes {
            node.process();
        }
    }
}

/// Counter for Braodcaster and Sink id generation
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Get the next unique id.
#[inline]
pub(in crate::frp) fn next_id() -> usize {
    ID_COUNTER.fetch_add(1, atomic::Ordering::Relaxed)
}
