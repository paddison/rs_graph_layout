mod graph_layout;

use std::collections::HashMap;

use env_logger::Env;
use graph_layout::GraphLayout;
use log::{debug, info};
use pyo3::prelude::*;
use rust_sugiyama::{C_MINIMIZATION_DEFAULT, RANKING_TYPE_DEFAULT};

pub type NodePositions = HashMap<usize, (isize, isize)>;

#[pyclass]
#[derive(Clone)]
pub struct SugiyamaConfig {
    #[pyo3(get, set)]
    vertex_size: isize,
    #[pyo3(get, set)]
    dummy_vertices: bool,
    #[pyo3(get, set)]
    dummy_size: f64,
    #[pyo3(get, set)]
    crossing_minimization: String,
    #[pyo3(get, set)]
    transpose: bool,
    #[pyo3(get, set)]
    layering_type: String,
}


#[pymethods]
impl SugiyamaConfig {
    #[new]
    #[pyo3(signature = (
            vertex_size=40,
            dummy_vertices=true,
            dummy_size=1.0,
            crossing_minimization=rust_sugiyama::C_MINIMIZATION_DEFAULT.into(),
            transpose=false,
            layering_type=rust_sugiyama::RANKING_TYPE_DEFAULT.into(),
            ))]
    fn new(
        vertex_size: isize,
        dummy_vertices: bool,
        dummy_size: f64,
        crossing_minimization: &str,
        transpose: bool,
        layering_type: &str,
    ) -> Self {
        Self {
            vertex_size,
            dummy_vertices,
            dummy_size,
            crossing_minimization: crossing_minimization.to_string(),
            transpose,
            layering_type: layering_type.to_string(),
        }
    }
}

impl From<SugiyamaConfig> for rust_sugiyama::Config {
    fn from(config: SugiyamaConfig) -> Self {
        Self {
            minimum_length: rust_sugiyama::MINIMUM_LENGTH_DEFAULT,
            vertex_spacing: config.vertex_size as usize * 4,
            dummy_size: config.dummy_size,
            dummy_vertices: config.dummy_vertices,
            c_minimization: config.crossing_minimization.try_into().unwrap_or(C_MINIMIZATION_DEFAULT),
            transpose: config.transpose,
            ranking_type: config.layering_type.try_into().unwrap_or(RANKING_TYPE_DEFAULT),
        }
    }
}

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
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace")).try_init();
    info!(target: "temanejo", "Original method: Got {} vertices and {} edges. Vertex size: {}", nodes.len(), edges.len(), vertex_size);
    debug!(target: "temanejo", "Vertices {:?}\nEdges: {:?}", nodes, edges);

    GraphLayout::create_layers(&nodes, &edges, vertex_size, global_tasks_in_first_row)
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
    config: SugiyamaConfig,
) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace")).try_init();
    info!(target: "temanejo", "Sugiyama's method: Got {} vertices and {} edges. Vertex size: {}", nodes.len(), edges.len(), config.vertex_size);
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
        .with_config(config.into())
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
    m.add_class::<SugiyamaConfig>()?;
    m.add_function(wrap_pyfunction!(create_layouts_original, m)?)?;
    m.add_function(wrap_pyfunction!(create_layouts_sugiyama, m)?)?;
    Ok(())
}
