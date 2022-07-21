use crate::frp::{
    inputs::StoreLast, Behaviour, BehaviourNode, Error, Event, FixedInputSet, FixedOutSet, IntoBehaviourNode,
    TypedInHandle,
};
use std::{fmt::Debug, marker::PhantomData};

pub struct InspectorPinLayout<T: Event + Debug> {
    pub input: TypedInHandle<T>,
}

#[derive(Default)]
pub struct Inspector<T: Event + Debug>(PhantomData<T>);

impl<T: Event + Debug> Behaviour for Inspector<T> {
    type InputSet = FixedInputSet<StoreLast<T>>;
    type OutputSet = FixedOutSet<()>;
    type PinLayout = InspectorPinLayout<T>;

    fn behave(&mut self, inputs: &mut Self::InputSet, _outputs: &mut Self::OutputSet) {
        // NO-PANIC: it should be called only after some event's been stored in the input.
        let input = &mut *inputs;
        let event = input.take().unwrap();
        log::trace!("{:?}", event)
    }

    fn get_pins(
        &self,
        input_set: &std::rc::Rc<std::cell::RefCell<Self::InputSet>>,
        _output_set: &std::rc::Rc<std::cell::RefCell<Self::OutputSet>>,
    ) -> Self::PinLayout {
        InspectorPinLayout {
            input: TypedInHandle::new(input_set, 0),
        }
    }
}

impl<T: Event + Debug> IntoBehaviourNode for Inspector<T> {
    type Behaviour = Self;

    fn into_behaviour_node(self) -> Result<BehaviourNode<Self::Behaviour>, Error> {
        let input_set = FixedInputSet::default();
        let output_set = FixedOutSet::default();
        Ok(BehaviourNode::new(input_set, output_set, self))
    }
}
