use crate::bench::lcg::LCG;
use std::time::SystemTime;

/*********************************************************
 *
 * Graph generators
 *
 */


/// A Layered Graph Generator is used to create a graph in
/// the form of:
/// 1    /---*---\
/// 2  /-*-\   /-*-\
/// 3  *-v-*   *-v-*
/// 4    *---v---*
/// 5        *
///     
/// It can be seen as two complete k-ary trees "glued" together
/// with the lower tree being upside down.
///
/// The graph is created by specifying an amount of n layers.
/// The above graph for example has n = 5 layers.
///
/// It is used to create a [`LayeredGraphRandomizer`], which can be used
/// to randomize the result graph.
/// Thus it can be either initialized with a seed, or let the seed
/// be set automatically.
/// Furthermore the degree of the graph can be specified, meaning
/// that a vertex will always have k outgoing (or incoming, depending
/// on the layer) edges.
///
/// Usage:
///
/// ```
/// // Create a GraphGenerator with 5 layers, a user defined seed of degree 3
/// let g: LayeredGraphRandomizer = LayeredGraphGenerator::new(5)
///     .with_seed(123456u128)
///     .with_degree(3);
/// ```
///
///
struct LayeredGraphGenerator {
    n: usize,
    seed: Option<u128>,
}

impl LayeredGraphGenerator {
    pub fn new(layers: usize) -> Self {
        Self {
            n: layers,
            seed: None,
        }
    }

    pub fn with_seed(mut self, seed: u128) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_degree(self, deg: usize) -> LayeredGraphRandomizer {
        // Divide graph into two halfs, and 'glue' them together
        let n_layers_half = self.n.div_ceil(2); // number of layers of one half
        let pow = deg.pow(n_layers_half as u32 - 1);

        // the first n vertices have n - 1 edges
        let n_edges_half = geo_series(deg, n_layers_half as u32) - 1;
        let total_vertices =
            2 * n_edges_half + 2 - ((self.n & 1) * deg.pow(n_layers_half as u32 - 1));

        let mut edges: Vec<_> = EdgesCalculator::new(deg).take(n_edges_half).collect();

        edges.extend(
            EdgesCalculator::new(deg)
                .take(n_edges_half)
                .map(|(j, i)| (total_vertices - i - 1, total_vertices - j - 1)),
        );

        if self.n % 2 == 0 {
            edges.extend((pow - deg + 1..).take(pow).map(|i| (i, i + pow)));
        }

        LayeredGraphRandomizer {
            n: self.n,
            k: deg,
            edges,
            n_vertices: total_vertices,
            lcg: match self.seed {
                Some(seed) => LCG::new_seed(seed),
                None => LCG::new(),
            },
        }
    }
}

/// A LayeredGraphRandomizer can be used to randomize a layered graph, created
/// by a [`LayeredGraphGenerator`].
///
/// There are four ways to do so:
/// 1. [`add_random_edge`]
/// 2. [`add_random_edges`]
/// 3. [`add_random_edge_in_layer`]
/// 4. [`add_random_edges_in_layer`]
///
/// These will add additional edges in the graph, likely increasing the amount
/// of edge crossings between the layers.
///
/// Layers are 'one' indexed and add an edge between the specified layer and 
/// the next one. For example to add a random edge between layer 2 and 3 specify
/// layer 2.
///
/// Adding an edge might fail, it the layer is already full, in which case 
/// the graph will not be altered.
///
/// Example Usage:
///
/// ```
/// // a randomizer can only be created through a LayeredGraphGenerator
/// let graph_randomizer = LayeredGraphGenerator::new().with_degree(2);
/// // randomize the graph and get the edges.
/// let edges: Vec<(usize, usize)> = graph_randomizer
///                 .add_random_edge()               // add a random edge on a random layer
///                 .add_random_edges_in_layer(2, 3) // add 2 random edges between layer 3 and 4
///                 .build();
/// ```
struct LayeredGraphRandomizer {
    n: usize, // number of layers
    k: usize, // degree
    edges: Vec<(usize, usize)>,
    n_vertices: usize,
    lcg: LCG,
}

impl LayeredGraphRandomizer {
    /// Build the graph, returning a vec of tuples, where each entry corresponds
    /// to an edge in the form of `(tail, head)`.
    pub fn build(self) -> Vec<(usize, usize)> {
        self.edges
    }

    /// Add a single random edge between two random layers
    pub fn add_random_edge(mut self) -> Self {
        let layer = self.lcg.generate_range(self.n);
        self.add_random_edge_in_layer(layer + 1) // this function assumes layers start at one
    }


    /// Add `amount` random edges in the graph on random layers
    pub fn add_random_edges(mut self, amount: usize) -> Self {
        for _ in 0..amount {
            self = self.add_random_edge();
        }
        self
    }

    /// Add a edge randomly in edges between layers "layer" and "layer" + 1
    /// "layer" has to be less than the amount of layers in the graph minus one
    /// and greater 1.
    /// Layers start from one
    ///
    /// Since edges are added randomly, and there is a maximum amount of edges
    /// that can be added, we simply try 100 iterations to add an edge.
    /// If no edge valid edges is found after 100 tries, simply return the graph
    /// as is.
    ///
    pub fn add_random_edge_in_layer(mut self, mut layer: usize) -> Self {
        // simply ignore invalid input
        if layer >= self.n - 1 || layer <= 1 {
            self
        } else {
            // first create edge, then handle case if it is on bottom half or not
            layer -= 1; // subtract one so it behaves as if zero indexed
                        //
            let upper_range = self.determine_node_range(self.determine_relative_layer(layer));
            let lower_range = self.determine_node_range(self.determine_relative_layer(layer + 1));

            // try do add an edge. it might be that the layer is already full
            // therefore try to add an edge only for a certain number of iterations
            for _ in 0..100 {
                let tail = self.create_random_vertex(upper_range);
                let head = self.create_random_vertex(lower_range);

                if !self.edges.contains(&(tail, head)) {
                    self.edges.push((tail, head));
                    break;
                }
            }
            self
        }
    }

    /// Add `amount` edges between layer `layer` and `layer + 1`
    pub fn add_random_edges_in_layer(mut self, amount: usize, layer: usize) -> Self {
        for _ in 0..amount {
            self = self.add_random_edge_in_layer(layer);
        }
        self
    }

    fn determine_relative_layer(&self, layer: usize) -> (usize, bool) {
        if layer < self.n.div_ceil(2) {
            (layer, false)
        } else {
            let mut ret = self.n / 2;
            ret = ((self.n - 1) / 2) - (ret as isize - layer as isize).abs() as usize;
            (ret, true)
        }
    }

    fn determine_node_range(&self, (layer, is_lower_half): (usize, bool)) -> (usize, usize) {
        // how many vertices are in that layer
        let n_vertices = self.k.pow(layer as u32);
        let mut start = geo_series(self.k, layer as u32);
        if is_lower_half {
            start = self.n_vertices - start - n_vertices;
        }
        (n_vertices, start)
    }

    #[inline(always)]
    fn create_random_vertex(&mut self, (n_vertices, start): (usize, usize)) -> usize {
        self.lcg.generate_range(n_vertices) + start
    }
}

#[test]
fn test_layered_graph_randomizer_determine_relative_layer_odd() {
    let lgr = LayeredGraphGenerator::new(5).with_degree(3);
    let actual: Vec<_> = (0..lgr.n)
        .map(|n| lgr.determine_relative_layer(n))
        .collect();
    let expected = vec![(0, false), (1, false), (2, false), (1, true), (0, true)];
    assert_eq!(actual, expected);
}

#[test]
fn test_layered_graph_randomizer_determine_relative_layer_even() {
    let lgr = LayeredGraphGenerator::new(6).with_degree(2);
    let actual: Vec<_> = (0..lgr.n)
        .map(|n| lgr.determine_relative_layer(n))
        .collect();
    let expected = vec![
        (0, false),
        (1, false),
        (2, false),
        (2, true),
        (1, true),
        (0, true),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn test_layered_graph_randomizer_add_random_edge_even() {
    // only check if it crashes or not
    let mut lgr = LayeredGraphGenerator::new(6).with_degree(2);
    println!("{:?}", lgr.edges);
    lgr = lgr.add_random_edge_in_layer(4);
    let actual = lgr.edges.last().unwrap();
    assert!(actual.0 <= 10 && actual.0 >= 7 && actual.1 <= 12 && actual.1 >= 11);
}

#[test]
fn determine_node_range_2edges_7layers_3() {
    let lgr = LayeredGraphGenerator::new(7).with_degree(2);
    // third layer
    let actual = lgr.determine_node_range((2, false));
    assert_eq!(actual, (4, 3));
}

#[test]
fn determine_node_range_2edges_7layers_4() {
    let lgr = LayeredGraphGenerator::new(7).with_degree(2);
    // third layer
    let actual = lgr.determine_node_range((2, true));
    println!("{}", lgr.n_vertices);
    assert_eq!(actual, (4, 15));
}

#[test]
fn determine_node_range_2edges_8layers_4() {
    let lgr = LayeredGraphGenerator::new(8).with_degree(2);
    // third layer
    let actual = lgr.determine_node_range((3, false));
    assert_eq!(actual, (8, 7));
}

#[test]
fn determine_node_range_2edges_8layers_5() {
    let lgr = LayeredGraphGenerator::new(8).with_degree(2);
    // third layer
    let actual = lgr.determine_node_range((3, true));
    assert_eq!(actual, (8, 15));
}

#[test]
fn determine_node_range_3edges_5layers_2() {
    let lgr = LayeredGraphGenerator::new(5).with_degree(3);
    // third layer
    let actual = lgr.determine_node_range((2, false));
    assert_eq!(actual, (9, 4));
}

#[test]
fn determine_node_range_3edges_6layers_4() {
    let lgr = LayeredGraphGenerator::new(6).with_degree(3);
    // third layer
    let actual = lgr.determine_node_range((2, true));
    assert_eq!(actual, (9, 13));
}

/*******************************************
 *
 * Helper functions and structs
 *
 */

/// Calculates the predecessor of the ith vertex in
/// a complete k-ary tree
struct EdgesCalculator {
    i: usize, // current vertex
    k: usize, // degree
}

impl EdgesCalculator {
    fn new(deg: usize) -> Self {
        Self { i: 1, k: deg }
    }
}

impl Iterator for EdgesCalculator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        let j = (i - 1) / self.k;
        self.i += 1;
        Some((j, i))
    }
}


/// Calculates sum i=0 to (n - 1)(k^i)
#[inline(always)]
fn geo_series(k: usize, n: u32) -> usize {
    (k.pow(n) - 1) / (k - 1)
}

#[test]
fn test_geo_series() {
    for k in 2usize..10 {
        let mut cur = 0;
        for n in 0..10 {
            assert_eq!(cur, geo_series(k, n));
            cur += k.pow(n);
        }
    }
}
