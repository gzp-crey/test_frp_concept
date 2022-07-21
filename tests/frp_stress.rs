#![feature(test)]
extern crate test;

use rand::Rng;
use frp::{
    frp::{
        inputs::StoreLast, Behaviour, BehaviourNode, Error, FixedInputSet, FixedOutSet, IntoBehaviourNode, Out, System,
        TypedInHandle, TypedOutHandle,
    },
    graph::{Node, Edge, Graph, DotAttribute}
};
use wasmer::{Store, Module, Instance, Value, imports};
use std::{cell::RefCell, rc::Rc, borrow::Cow};
use test::Bencher;

pub struct PinLayout {
    pub in1: TypedInHandle<f64>,
    pub in2: TypedInHandle<f64>,
    pub output: TypedOutHandle<f64>,
}

struct NodeData(String);
impl DotAttribute for NodeData {
    fn label(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.0))
    }
}

struct EdgeData(String);
impl DotAttribute for EdgeData {
    fn label(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.0))
    }

    fn font_size(&self) -> Option<u32> {
        Some(8)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Min,
    Max,
    Avg,
}

pub struct Operation(Op);

impl Behaviour for Operation {
    type InputSet = FixedInputSet<(StoreLast<f64>, StoreLast<f64>)>;
    type OutputSet = FixedOutSet<Out<f64>>;
    type PinLayout = PinLayout;

    fn behave(&mut self, input_set: &mut Self::InputSet, output_set: &mut Self::OutputSet) {
        let mut rng = rand::thread_rng();
        let (i1, i2) = &**input_set;
        let i1 = i1.try_get().cloned().unwrap_or_else(|| rng.gen_range(0.0..100.0));
        let i2 = i2.try_get().cloned().unwrap_or_else(|| rng.gen_range(0.0..100.0));
        let output = &mut **output_set;

        let val = match self.0 {
            Op::Add => i1 + i2,
            Op::Sub => i1 - i2,
            Op::Mul => i1 * i2,
            Op::Min => {
                if i1 < i2 {
                    i1
                } else {
                    i2
                }
            }
            Op::Max => {
                if i1 > i2 {
                    i1
                } else {
                    i2
                }
            }
            Op::Avg => (i1 + i2) / 2.,
        };
        output.send(&val);
    }

    fn get_pins(
        &self,
        input_set: &Rc<RefCell<Self::InputSet>>,
        output_set: &Rc<RefCell<Self::OutputSet>>,
    ) -> Self::PinLayout {
        PinLayout {
            in1: TypedInHandle::new(input_set, 0),
            in2: TypedInHandle::new(input_set, 1),
            output: TypedOutHandle::new(output_set, 0),
        }
    }
}

impl IntoBehaviourNode for Operation {
    type Behaviour = Self;

    fn into_behaviour_node(self) -> Result<BehaviourNode<Self::Behaviour>, Error> {
        let input_set = FixedInputSet::default();
        let output_set = FixedOutSet::default();
        Ok(BehaviourNode::new(input_set, output_set, self))
    }
}



pub struct WasmScript{
    instance: Instance
}

impl WasmScript {
    pub fn new(script: &str) -> Self {
        let store = Store::default();
        let module = Module::new(&store, script).unwrap();    
        let import_object = imports! {};
        let instance = Instance::new(&module, &import_object).unwrap();
        Self {instance}
    }
}

impl Behaviour for WasmScript {
    type InputSet = FixedInputSet<(StoreLast<f64>, StoreLast<f64>)>;
    type OutputSet = FixedOutSet<Out<f64>>;
    type PinLayout = PinLayout;

    fn behave(&mut self, input_set: &mut Self::InputSet, output_set: &mut Self::OutputSet) {
        let mut rng = rand::thread_rng();
        let (i1, i2) = &**input_set;
        let i1 = i1.try_get().cloned().unwrap_or_else(|| rng.gen_range(0.0..100.0));
        let i2 = i2.try_get().cloned().unwrap_or_else(|| rng.gen_range(0.0..100.0));
        let output = &mut **output_set;

        let p = i1+i2;
        
        let behave = self.instance.exports.get_function("add_one").unwrap();
        let result = behave.call(&[Value::F64(p)]).unwrap();
        let value = match &result[0] {
            Value::F64(p) => *p,
            _ => -1.,
        };
        output.send(&value);
    }

    fn get_pins(
        &self,
        input_set: &Rc<RefCell<Self::InputSet>>,
        output_set: &Rc<RefCell<Self::OutputSet>>,
    ) -> Self::PinLayout {
        PinLayout {
            in1: TypedInHandle::new(input_set, 0),
            in2: TypedInHandle::new(input_set, 1),
            output: TypedOutHandle::new(output_set, 0),
        }
    }
}


impl IntoBehaviourNode for WasmScript {
    type Behaviour = Self;

    fn into_behaviour_node(self) -> Result<BehaviourNode<Self::Behaviour>, Error> {
        let input_set = FixedInputSet::default();
        let output_set = FixedOutSet::default();
        Ok(BehaviourNode::new(input_set, output_set, self))
    }
}

#[bench]
fn bench_run(b: &mut Bencher) {
    const NODES: usize = 20;
    const EDGES: usize = 500;

    let mut system = System::default();
    let input = system.create_input::<f64>();

    let mut gnodes = Vec::new();
    let mut gedges = Vec::new();

    let mut script_node = 0;
    let mut rng = rand::thread_rng();
    let mut nodes = Vec::new();
    let mut connected = Vec::new();
    for _ in 0..NODES {
        let (name, pin_layout) = if rng.gen_bool(0.7) {
            let module_wat = r#"
            (module
              (type $t0 (func (param f64) (result f64)))
              (func $add_one (export "add_one") (type $t0) (param $p0 f64) (result f64)
                get_local $p0
                f64.const 1
                f64.add))
            "#;

            script_node += 1;
            ("wasm".to_string(), system.add_behaviour(WasmScript::new(module_wat)).unwrap())
        }
        else {
            let op = match rng.gen_range(0u8..6) {
                0 => Op::Add,
                1 => Op::Sub,
                2 => Op::Mul,
                3 => Op::Min,
                4 => Op::Max,
                _ => Op::Avg,
            };

            (format!("{:?}", op), system.add_behaviour(Operation(op)).unwrap())
        };
        nodes.push(pin_layout);
        connected.push((false,false));

        gnodes.push(Node{data: NodeData(name)});
    }
    // system input
    gnodes.push(Node{data: NodeData("INPUT".to_string())});

    let mut edge_count = 0;
    for _ in 0..EDGES {
        let from = rng.gen_range(0usize..nodes.len());
        let to = rng.gen_range(0usize..nodes.len());
        let (from, to) = if from > to {
            (to, from)
        } else if from < to {
            (from, to)
        } else {
            continue;
        };
                
        if rng.gen_bool(0.5) { 
            if connected[to].0 {
                continue;
            }
            system.connect(&nodes[from].output, &nodes[to].in1).unwrap();
            gedges.push(Edge{
                data: EdgeData("P1".to_string()),
                from,
                to,
            });
            connected[to].0 = true;
            edge_count += 1;
        } else {
            if connected[to].1 {
                continue;
            }
            system.connect(&nodes[from].output, &nodes[to].in2).unwrap();
            gedges.push(Edge{
                data: EdgeData("P2".to_string()),
                from,
                to,
            });
            connected[to].1 = true;
            edge_count += 1;
        }
    }

    let mut incount = 0;
    for (n, (a,b)) in connected.into_iter().enumerate() {
        if !a {
            system.connect(&input, &nodes[n].in1).unwrap();
            gedges.push(Edge{
                data: EdgeData("P1".to_string()),
                from: nodes.len(),
                to: n,
            });
            incount += 1;
        }

        if !b {
            system.connect(&input, &nodes[n].in2).unwrap();
            gedges.push(Edge{
                data: EdgeData("P2".to_string()),
                from: nodes.len(),
                to: n,
            });
            incount += 1;
        }
    }

    let ggraph = Graph{nodes: gnodes, edges: gedges};
    println!("{}", ggraph.dot_graph());    

    println!("node: {}", nodes.len());
    println!("script node: {}", script_node);    
    println!("input edge count: {}", incount);
    println!("edge count: {}", edge_count);

    b.iter(move || {
        let mut rng = rand::thread_rng();
        let v = rng.gen_range(0.0..100.0);
        system.run_on(input.clone(), &v).unwrap();
    });
}
