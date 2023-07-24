# rs_graph_layout

## Description

Creates a layered graph layout from a list of edges for the Temanejo Debugger.

## Usage

1. Compile: `cargo build --release`
2. Move the file to the python dir:`mv target/release/librs_graph_layout.[dll, dylib, so] /path/to/python/file/rs_graph_layout.so`
3. In the python file:
```
import rs_graph_layout as rs

edges = [(1, 1), .... (12, 15)]
node_size = 40
global_tasks_in_first_row = False
layout = rs.create_layouts_i32(edges, node_size, global_tasks_in_first_row)

print(layout)
