mod graph_layout;

use std::collections::HashMap;

use env_logger::Env;
use graph_layout::GraphLayout;
use log::{debug, info};
use pyo3::prelude::*;
use rust_sugiyama::{CrossingMinimization, RankingType};

pub type NodePositions = HashMap<usize, (isize, isize)>;

/// Create the layouts for each weakly connected component contained in edges.
///
/// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
/// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
/// of the graph.
/// This version uses the original method of Temanejo to calculate the coordinates.
#[pyfunction]
pub fn create_layouts_original(
    nodes: Vec<u32>,
    edges: Vec<(u32, u32)>,
    vertex_size: isize,
    global_tasks_in_first_row: bool,
) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!(target: "temanejo", "Original method: Got {} vertices and {} edges. Vertex size: {}", nodes.len(), edges.len(), vertex_size);
    debug!(target: "temanejo", "Vertices {:?}\nEdges: {:?}", nodes, edges);

    GraphLayout::<isize>::create_layers(&nodes, &edges, vertex_size, global_tasks_in_first_row)
}

/// Create the layouts for each weakly connected component contained in edges.
///
/// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
/// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
/// This version uses Suiyama's method to calculate the coordinates.
#[pyfunction]
pub fn create_layouts_sugiyama(
    mut nodes: Vec<u32>,
    mut edges: Vec<(u32, u32)>,
    vertex_size: isize,
    dummy_vertices: bool,
    dummy_size: f64,
    crossing_minimization: String,
    transpose: bool,
    layering_type: String,
) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!(target: "temanejo", "Sugiyama's method: Got {} vertices and {} edges. Vertex size: {}", nodes.len(), edges.len(), vertex_size);
    debug!(target: "temanejo", "Vertices {:?}\nEdges: {:?}", nodes, edges);
    let mut layout_list = Vec::new();
    let mut width_list = Vec::new();
    let mut height_list = Vec::new();

    // decrement edges and nodes by one since networkx graph is 1 based.
    nodes.iter_mut().for_each(|v| *v -= 1);
    edges.iter_mut().for_each(|(t, h)| {
        *t -= 1;
        *h -= 1;
    });

    let layouts = rust_sugiyama::from_vertices_and_edges(&nodes, &edges)
        .vertex_spacing(vertex_size as usize * 4)
        .dummy_vertices(dummy_vertices)
        .dummy_size(dummy_size)
        .crossing_minimization(
            crossing_minimization
                .try_into()
                .unwrap_or(CrossingMinimization::Median),
        )
        .transpose(transpose)
        .layering_type(layering_type.try_into().unwrap_or(RankingType::Up))
        .build();

    for (layout, width, height) in layouts {
        width_list.push(width);
        height_list.push(height);
        layout_list.push(HashMap::<usize, (isize, isize)>::from_iter(
            layout.into_iter().map(|(id, coords)| (id + 1, coords)),
        ));
    }

    (layout_list, width_list, height_list)
}

#[pymodule]
fn rs_graph_layout(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_layouts_original, m)?)?;
    m.add_function(wrap_pyfunction!(create_layouts_sugiyama, m)?)?;
    Ok(())
}
