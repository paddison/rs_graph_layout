mod graph_layout;

use graph_layout::{ GraphLayout, NodePositions };
use pyo3::prelude::*;

/// Create the layouts for each weakly connected component contained in edges.
/// 
/// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
/// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
/// of the graph.
/// This is the version where the data of the nodes will be a i32 integer.
#[pyfunction]
pub fn create_layouts_i32(nodes: Vec<u32>, edges: Vec<(u32, u32)>, node_size: isize, global_tasks_in_first_row: bool) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    println!("nodes: {:?}\nedges: {:?}\n {}", nodes, edges, global_tasks_in_first_row);
    GraphLayout::<i32>::create_layers(&nodes, &edges, node_size, global_tasks_in_first_row)
}

#[pymodule]
fn  rs_graph_layout(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_layouts_i32, m)?)?;
    Ok(())
}