# poc for finctional reactive program evaluation

The "program" is a graph where a node is executed when there was any change on the input.
In the POC:
- all node has 2 f64 input and 1 f64 output.
- the "native" nodes perform one of +,-,*,min,max,avg operation
- the wasm is a brute fore, all node store the whole wasm instance with its store, etc. The script is the same: (a+b)+1, jsut a quik check for wasm interop "cost"

Runing:
- test: `cargo test`
- benchmark: `cargo bench --test frp_stress -- --nocapture`

to check the dot: 
- <https://graphviz.org/download/>
- save the output into some file
- `dot -Tps filename.dot -o outfile.ps -v`
- waaaaaait 1-2hour, maybe day depending on the size :)
