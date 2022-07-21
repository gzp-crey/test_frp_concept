# poc for finctional reactive program evaluation

run test: `cargo test`
run banch: `cargo bench --test frp_stress -- --nocapture`


to check the dot: 
- <https://graphviz.org/download/>
- save the output into some file
- `dot -Tps filename.dot -o outfile.ps -v`
- waaaaaait