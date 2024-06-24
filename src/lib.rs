/*
 AYUDAME/TEMANEJO toolset
--------------------------

 (C) 2024, HLRS, University of Stuttgart
 All rights reserved.
 This software is published under the terms of the BSD license:

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the <organization> nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

pub mod graph_layout;

use std::collections::HashMap;

use env_logger::Env;
use graph_layout::GraphLayout;
use log::{debug, info};
use pyo3::prelude::*;
use rust_sugiyama::configure::{C_MINIMIZATION_DEFAULT, RANKING_TYPE_DEFAULT};

pub type NodePositions = HashMap<usize, (isize, isize)>;

/// Can be used to configure Sugiyama's algorithm.
///
/// Seef [rust_sugiyama::configure::Config] for more information.
#[pyclass]
#[derive(Clone)]
pub struct SugiyamaConfig {
    /// Size of the vertices
    #[pyo3(get, set)]
    vertex_size: isize,
    /// use dummy vertices
    #[pyo3(get, set)]
    dummy_vertices: bool,
    /// size of dummy vertices
    #[pyo3(get, set)]
    dummy_size: f64,
    /// Which heuristic to use for crossing minimization.
    /// permitted values are: `barycenter` and `median`.
    #[pyo3(get, set)]
    crossing_minimization: String,
    /// Use the transpose function during crossing minimization.
    /// May lead to fewer edge crossing but drastically increases runtime of the algorithm
    #[pyo3(get, set)]
    transpose: bool,
    /// The method used to calculate the layer of each vertex. Permitted values are:
    /// - `minimize`: try to minimize edge length
    /// - `original`: use the original method of Temanejo
    /// - `up`: move vertices as far up as possible
    /// - `down`: move vertices as far down as possible
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
            crossing_minimization=rust_sugiyama::configure::C_MINIMIZATION_DEFAULT.into(),
            transpose=false,
            layering_type=rust_sugiyama::configure::RANKING_TYPE_DEFAULT.into(),
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

impl Default for SugiyamaConfig {
    fn default() -> Self {
        Self {
            vertex_size: 40,
            dummy_vertices: true,
            dummy_size: 1.0,
            crossing_minimization: <&'static str>::from(C_MINIMIZATION_DEFAULT).to_string(),
            transpose: false,
            layering_type: <&str>::from(RANKING_TYPE_DEFAULT).to_string(),
        }
    }
}

impl From<SugiyamaConfig> for rust_sugiyama::configure::Config {
    fn from(config: SugiyamaConfig) -> Self {
        Self {
            minimum_length: rust_sugiyama::configure::MINIMUM_LENGTH_DEFAULT,
            vertex_spacing: config.vertex_size as usize * 4,
            dummy_size: config.dummy_size,
            dummy_vertices: config.dummy_vertices,
            c_minimization: config
                .crossing_minimization
                .try_into()
                .unwrap_or(C_MINIMIZATION_DEFAULT),
            transpose: config.transpose,
            ranking_type: config
                .layering_type
                .try_into()
                .unwrap_or(RANKING_TYPE_DEFAULT),
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
#[allow(deprecated)]
fn rs_graph_layout(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<SugiyamaConfig>()?;
    m.add_function(wrap_pyfunction!(create_layouts_original, m)?)?;
    m.add_function(wrap_pyfunction!(create_layouts_sugiyama, m)?)?;
    Ok(())
}
