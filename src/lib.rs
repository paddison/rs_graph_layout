//mod graph_layout;

use std::collections::HashMap;

use env_logger::Env;
use log::{debug, info};
use pyo3::prelude::*;

pub type NodePositions = HashMap<usize, (isize, isize)>;
/// Create the layouts for each weakly connected component contained in edges.
///
/// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
/// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
/// of the graph.
/// This is the version where the data of the nodes will be a i32 integer.
#[pyfunction]
pub fn create_layouts_i32(
    mut nodes: Vec<u32>,
    mut edges: Vec<(u32, u32)>,
    vertex_size: isize,
    global_tasks_in_first_row: bool,
) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!(target: "temanejo", "Got {} vertices and {} edges. Vertex size: {}", nodes.len(), edges.len(), vertex_size);
    debug!(target: "temanejo", "Vertices {:?}\nEdges: {:?}", nodes, edges);

    // decrement edges and nodes by one since networkx graph is 1 based.
    nodes.iter_mut().for_each(|v| *v -= 1);
    edges.iter_mut().for_each(|(t, h)| {
        *t -= 1;
        *h -= 1;
    });
    let layouts = rust_sugiyama::from_vertices_and_edges(&nodes, &edges)
        .vertex_spacing(vertex_size as usize * 4)
        .dummy_vertices(false)
        .dummy_size(0.3)
        .crossing_minimization(rust_sugiyama::CrossingMinimization::Median)
        .transpose(true)
        .layering_type(rust_sugiyama::RankingType::Up)
        .build();

    let mut all_positions = Vec::new();
    let mut width_list = Vec::new();
    let mut height_list = Vec::new();
    for (layout, width, height) in layouts {
        let mut positions = HashMap::new();
        width_list.push(width);
        height_list.push(height);
        for (id, (x, y)) in layout {
            // don't forget to increment id again to make it fit for networkx
            positions.insert(id + 1, (x, y));
        }
        all_positions.push(positions);
    }
    return (all_positions, width_list, height_list);
}

#[pymodule]
fn rs_graph_layout(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_layouts_i32, m)?)?;
    Ok(())
}
