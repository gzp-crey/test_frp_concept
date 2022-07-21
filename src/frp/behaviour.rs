use crate::frp::{Error, Event, InputSet, OutputSet, TypedOutHandle};
use std::{cell::RefCell, rc::Rc};

/// Implements the core logic to consume input and generate output
pub trait Behaviour: 'static {
    type InputSet: InputSet;
    type OutputSet: OutputSet;
    type PinLayout;

    /// Perform the logic to consume inputs and trigger the outputs.
    fn behave(&mut self, input_set: &mut Self::InputSet, output_set: &mut Self::OutputSet);

    /// Return int input/output pin layout for clients to connect behaviour into graph.
    fn get_pins(
        &self,
        input_set: &Rc<RefCell<Self::InputSet>>,
        output_set: &Rc<RefCell<Self::OutputSet>>,
    ) -> Self::PinLayout;
}

/// Behaviour with the input and output sets.
pub struct BehaviourNode<B: Behaviour> {
    pub(in crate::frp) input_set: Rc<RefCell<<B as Behaviour>::InputSet>>,
    pub(in crate::frp) output_set: Rc<RefCell<<B as Behaviour>::OutputSet>>,
    behaviour: B,
}

impl<B: Behaviour> BehaviourNode<B> {
    pub fn new(input_set: <B as Behaviour>::InputSet, output_set: <B as Behaviour>::OutputSet, behaviour: B) -> Self {
        Self {
            input_set: Rc::new(RefCell::new(input_set)),
            output_set: Rc::new(RefCell::new(output_set)),
            behaviour,
        }
    }

    pub fn get_pins(&self) -> <B as Behaviour>::PinLayout {
        self.behaviour.get_pins(&self.input_set, &self.output_set)
    }
}

pub trait IntoBehaviourNode {
    type Behaviour: Behaviour;

    fn into_behaviour_node(self) -> Result<BehaviourNode<Self::Behaviour>, Error>;
}

/// Type erased `BehaviourNode`.
pub(in crate::frp) trait GeneralBehaviourNode {
    fn process(&mut self);
}

impl<B> GeneralBehaviourNode for BehaviourNode<B>
where
    B: Behaviour,
{
    fn process(&mut self) {
        // The input and output are borrowed for the entire process,
        // but since graph shall contain no cycle and hence no output shall
        // trigger the already borrowed input.
        let input = &mut *self.input_set.borrow_mut();
        let output = &mut *self.output_set.borrow_mut();
        if input.is_dirty() {
            input.reset_dirty();
            self.behaviour.behave(input, output);
        }
    }
}

pub enum BehaviourInput<T: Event> {
    /// Internal to the system
    Internal,
    /// Connect to the input of the system
    System(TypedOutHandle<T>),
}
