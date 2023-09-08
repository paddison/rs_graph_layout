mod graph_layout;

use std::collections::HashMap;

use pyo3::prelude::*;

pub type NodePositions = HashMap<usize, (isize, isize)>;
/// Create the layouts for each weakly connected component contained in edges.
/// 
/// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
/// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
/// of the graph.
/// This is the version where the data of the nodes will be a i32 integer.
#[pyfunction]
pub fn create_layouts_i32(nodes: Vec<u32>, edges: Vec<(u32, u32)>, node_size: isize, global_tasks_in_first_row: bool) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    println!("nodes: {:?}\nedges: {:?}\n {}", nodes, edges, global_tasks_in_first_row);
    let layouts = rust_sugiyama::algorithm::build_layout_from_vertices_and_edges(&nodes, &edges, 1, node_size as usize * 4);
    let mut all_positions = Vec::new();
    let mut width_list = Vec::new();
    let mut height_list = Vec::new();
    for (layout, width, height) in layouts {
        let mut positions = HashMap::new();
        width_list.push(width);
        height_list.push(height);
        for (id, (x, y)) in layout {
            positions.insert(id, (x, -y));    
        }
        all_positions.push(positions);
    }
    return (all_positions, width_list, height_list)
}

#[pymodule]
fn  rs_graph_layout(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_layouts_i32, m)?)?;
    Ok(())
}