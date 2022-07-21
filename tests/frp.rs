use frp::{
    frp::{
        behaviours::Inspector, inputs::StoreLast, Behaviour, BehaviourNode, Error, FixedInputSet, FixedOutSet,
        IntoBehaviourNode, Out, System, TypedInHandle, TypedOutHandle,
    }
};
use std::{cell::RefCell, rc::Rc};

pub struct StringDublicatorPinLayout {
    pub input: TypedInHandle<String>,
    pub output: TypedOutHandle<String>,
}

#[derive(Default)]
pub struct StringDublicator;

impl Behaviour for StringDublicator {
    type InputSet = FixedInputSet<StoreLast<String>>;
    type OutputSet = FixedOutSet<Out<String>>;
    type PinLayout = StringDublicatorPinLayout;

    fn behave(&mut self, input_set: &mut Self::InputSet, output_set: &mut Self::OutputSet) {
        let input = &**input_set.get();
        let output = &mut **output_set;
        output.send(&format!("{}{}", input, input));
    }

    fn get_pins(
        &self,
        input_set: &Rc<RefCell<Self::InputSet>>,
        output_set: &Rc<RefCell<Self::OutputSet>>,
    ) -> Self::PinLayout {
        StringDublicatorPinLayout {
            input: TypedInHandle::new(input_set, 0),
            output: TypedOutHandle::new(output_set, 0),
        }
    }
}

impl IntoBehaviourNode for StringDublicator {
    type Behaviour = Self;

    fn into_behaviour_node(self) -> Result<BehaviourNode<Self::Behaviour>, Error> {
        let input_set = FixedInputSet::default();
        let output_set = FixedOutSet::default();
        Ok(BehaviourNode::new(input_set, output_set, self))
    }
}

#[test]
fn simple() {
    let mut system = System::default();
    let input = system.create_input::<String>();

    let string_dup = system.add_behaviour(StringDublicator::default()).unwrap();
    let inspect = system.add_behaviour(Inspector::<String>::default()).unwrap();

    system.connect(&input, &string_dup.input).unwrap();
    system.connect(&string_dup.output, &inspect.input).unwrap();

    //log::trace!("{}", system.get_dot_graph(GraphDetail::Whole));
    system.run_on(input, &"Hello World".to_string()).unwrap();
}
