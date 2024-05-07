use std::time::SystemTime;
use crate::bench::lcg::LCG;

// It may be a good idea to store edges in layer corresponding
// to if they are contained in the lower, upper or middle layers
struct LayeredGraphGenerator {
    n: usize,
}

impl LayeredGraphGenerator {
    pub fn new(layers: usize) -> Self {
        Self { n: layers }
    }

    pub fn add_edges(self, deg: usize) -> LayeredGraphRandomizer {
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
        }
    }
}

struct LayeredGraphRandomizer {
    n: usize, // number of layers
    k: usize, // degree
    edges: Vec<(usize, usize)>,
    n_vertices: usize,
}

impl LayeredGraphRandomizer {
    pub fn build(self) -> Vec<(usize, usize)> {
        self.edges
    }

    pub fn add_random_edge(self) -> Self {
        let layer = LCG::new().generate_range(self.n);
        self.add_random_edge_in_layer(layer)
    }

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
    pub fn add_random_edge_in_layer(mut self, mut layer: usize) -> Self {
        // todos think about how to adjust layers so it works
        // simply ignore invalid input
        if layer >= self.n - 1 || layer <= 1 {
            self
        } else {
            // first create edge, then handle case if it is on bottom half or not
            let mut lcg = LCG::new();
            layer -= 1; // subtract one so it behaves as if zero indexed
                        // determine the layers we're acting on
            let upper_range = self.determine_node_range(self.determine_relative_layer(layer));
            let lower_range = self.determine_node_range(self.determine_relative_layer(layer + 1));

            // try do add an edge. it might be that the layer is already full
            // therefore try to add an edge only for a certain number of iterations
            for _ in 0..100 {
                let tail = self.create_random_vertex(upper_range, &mut lcg);
                let head = self.create_random_vertex(lower_range, &mut lcg);

                if !self.edges.contains(&(tail, head)) {
                    self.edges.push((tail, head));
                    break;
                }
            }
            self
        }
    }

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
        let n_vertices = 2usize.pow(layer as u32);
        let mut start = n_vertices - 1;
        let bool = true;
        match bool {
            true => (),
            false => (),
        }
        if is_lower_half {
            start = self.n_vertices - start - n_vertices;
        }
        (n_vertices, start)
    }

    #[inline(always)]
    fn create_random_vertex(&self, (n_vertices, start): (usize, usize), lcg: &mut LCG) -> usize {
        lcg.generate_range(n_vertices) + start
    }
}

#[test]
fn test_layered_graph_randomizer_determine_relative_layer_odd() {
    let lgr = LayeredGraphGenerator::new(5).add_edges(3);
    let actual: Vec<_> = (0..lgr.n)
        .map(|n| lgr.determine_relative_layer(n))
        .collect();
    let expected = vec![(0, false), (1, false), (2, false), (1, true), (0, true)];
    assert_eq!(actual, expected);
}

#[test]
fn test_layered_graph_randomizer_determine_relative_layer_even() {
    let lgr = LayeredGraphGenerator::new(6).add_edges(2);
    let actual: Vec<_> = (0..lgr.n)
        .map(|n| lgr.determine_relative_layer(n))
        .collect();
    let expected = vec![(0, false), (1, false), (2, false), (2, true), (1, true), (0, true)];
    assert_eq!(actual, expected);
}

#[test]
fn test_layered_graph_randomizer_add_random_edge_even() {
    // only check if it crashes or not
    let mut lgr = LayeredGraphGenerator::new(6).add_edges(2);
    println!("{:?}", lgr.edges);
    lgr = lgr.add_random_edge_in_layer(4);
    let actual = lgr.edges.last().unwrap();
    assert!(actual.0 <= 10 && actual.0 >= 7 && actual.1 <= 12 && actual.1 >= 11);
}

#[test]
fn determine_node_range_2edges_7layers_3() {
    let lgr = LayeredGraphGenerator::new(7).add_edges(2);
    // third layer
    let actual = lgr.determine_node_range((2, false)); 
    assert_eq!(actual, (4, 3));
}

#[test]
fn determine_node_range_2edges_7layers_4() {
    let lgr = LayeredGraphGenerator::new(7).add_edges(2);
    // third layer
    let actual = lgr.determine_node_range((2, true)); 
    assert_eq!(actual, (4, 15));
}

#[test]
fn determine_node_range_2edges_8layers_4() {
    let lgr = LayeredGraphGenerator::new(8).add_edges(2);
    // third layer
    let actual = lgr.determine_node_range((3, false)); 
    assert_eq!(actual, (8, 7));
}

#[test]
fn determine_node_range_2edges_8layers_5() {
    let lgr = LayeredGraphGenerator::new(8).add_edges(2);
    // third layer
    let actual = lgr.determine_node_range((3, true)); 
    assert_eq!(actual, (8, 15));
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

