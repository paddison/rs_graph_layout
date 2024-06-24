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
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use petgraph::{
    algo::toposort,
    stable_graph::{NodeIndex, StableDiGraph},
    visit::IntoNodeIdentifiers,
    Direction,
};

use super::NodePositions;

/// Represents a layout of a graph.
/// The nodes of the graph are arranged in layers.
///
/// The fields are:
///     - graph: the actual graph
///     - layers: the layers containing the nodes of the graph
///     - level_of_node: the current level of a node
///     - ndex_of_node: the index of a node in its level
///     - node_size: the size of a node when drawn in pixel
///     - node_separation: the minimum separation of two nodes
///     - global_tasks_in_first_row: boolean, indicating if global tasks need to be put in the first row  
#[derive(Debug)]
pub struct GraphLayout {
    graph: StableDiGraph<(), ()>,
    layers: RefCell<Vec<Vec<Option<NodeIndex>>>>,
    level_of_node: RefCell<HashMap<NodeIndex, usize>>,
    index_of_node: RefCell<HashMap<NodeIndex, usize>>,
    _node_size: isize,
    node_separation: isize,
    global_tasks_in_first_row: bool,
}

impl GraphLayout {
    /// Create the layouts for each weakly connected component contained in edges.
    ///
    /// A layout contains the position of each node (HashMap of NodeIndex and (x, y)) the height of the layout and the maximum width of the layers.
    /// The layout is created by arranging the nodes of the graph in level and performing some operations them in order to produce a visualization
    /// of the graph.
    pub fn create_layers(
        nodes: &[u32],
        edges: &[(u32, u32)],
        node_size: isize,
        global_tasks_in_first_row: bool,
    ) -> (Vec<NodePositions>, Vec<usize>, Vec<usize>) {
        let mut layout_list = Vec::new();
        let mut width_list = Vec::new();
        let mut height_list = Vec::new();
        let mut graph = StableDiGraph::<(), ()>::new();

        for _ in nodes {
            graph.add_node(());
        }

        for (predecessor, successor) in edges {
            // networkx graph is 1 indexed
            graph.add_edge(
                NodeIndex::from(*predecessor - 1),
                NodeIndex::from(*successor - 1),
                (),
            );
        }

        let mut graphs = Self::into_weakly_connected_components(graph)
            .into_iter()
            .map(|subgraph| Self::new(subgraph, node_size, global_tasks_in_first_row))
            .collect::<Vec<_>>();

        for graph in graphs.iter_mut() {
            if graph.graph.edge_count() != 0 {
                graph.align_nodes();
            }
        }

        for (node_positions, width, height) in graphs.into_iter().map(|graph| graph.build_layout())
        {
            layout_list.push(node_positions);
            width_list.push(width);
            height_list.push(height);
        }

        (layout_list, width_list, height_list)
    }

    fn build_layout_no_edges(&self) -> (NodePositions, usize, usize) {
        let node = self.graph.node_indices().next().unwrap();
        // increment node index by one for networkx
        (
            HashMap::from([(node.index() + 1, (self.node_separation, 0))]),
            1,
            1,
        )
    }

    fn build_layout(&self) -> (NodePositions, usize, usize) {
        if self.graph.edge_count() == 0 {
            return self.build_layout_no_edges();
        }
        let mut node_positions = HashMap::new();
        let offset = if self.layers.borrow()[0].iter().all(|n| n.is_none()) {
            1
        } else {
            0
        };

        for (level_index, level) in self.layers.borrow().iter().enumerate() {
            for (node_index, node_opt) in level.iter().enumerate() {
                let node = if let Some(node) = node_opt {
                    *node
                } else {
                    continue;
                };
                let x = node_index as isize * self.node_separation;
                let y = (-(level_index as isize) + offset) * self.node_separation;
                node_positions.insert(node.index() + 1, (x, y)); // increment index by one for networkx
            }
        }
        (node_positions, self.get_width(), self.get_nums_of_level())
    }

    /// Takes a graph and breaks it down into its weakly connected components.
    /// A weakly connected component is a list of edges which are connected with each other.
    fn into_weakly_connected_components(
        graph: StableDiGraph<(), ()>,
    ) -> Vec<StableDiGraph<(), ()>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();

        for node in graph.node_indices() {
            if visited.contains(&node) {
                continue;
            }

            let component_nodes = Self::component_dfs(node, &graph);
            let component = graph.filter_map(
                |n, _| {
                    if component_nodes.contains(&n) {
                        Some(())
                    } else {
                        None
                    }
                },
                |_, _| Some(()),
            );

            component_nodes.into_iter().for_each(|n| {
                visited.insert(n);
            });
            components.push(component);
        }

        components
    }

    fn component_dfs(start: NodeIndex, graph: &StableDiGraph<(), ()>) -> HashSet<NodeIndex> {
        let mut queue = vec![start];
        let mut visited = HashSet::new();

        visited.insert(start);

        while let Some(cur) = queue.pop() {
            for neighbor in graph.neighbors_undirected(cur) {
                if visited.contains(&neighbor) {
                    continue;
                }
                visited.insert(neighbor);
                queue.push(neighbor);
            }
        }

        visited
    }

    fn new(
        graph: StableDiGraph<(), ()>,
        node_size: isize,
        global_tasks_in_first_row: bool,
    ) -> Self {
        Self {
            graph,
            level_of_node: RefCell::new(HashMap::new()),
            index_of_node: RefCell::new(HashMap::new()),
            layers: RefCell::new(Vec::new()),
            _node_size: node_size,
            node_separation: node_size * 4,
            global_tasks_in_first_row,
        }
    }

    fn get_level_of_node(&self, node: &NodeIndex) -> Option<usize> {
        self.level_of_node.borrow().get(node).cloned()
    }

    fn insert_level_of_node(&self, node: NodeIndex, level: usize) -> Option<usize> {
        self.level_of_node.borrow_mut().insert(node, level)
    }

    fn get_index_of_node(&self, node: &NodeIndex) -> Option<usize> {
        self.index_of_node.borrow().get(node).cloned()
    }

    fn insert_index_of_node(&self, node: NodeIndex, index: usize) -> Option<usize> {
        self.index_of_node.borrow_mut().insert(node, index)
    }

    fn get_nums_of_level(&self) -> usize {
        let mut num_levels = 0;
        for layer in self.layers.borrow().iter() {
            if layer.iter().any(|n| n.is_some()) {
                num_levels += 1;
            }
        }
        num_levels
    }

    fn get_width(&self) -> usize {
        self.layers
            .borrow()
            .iter()
            .map(|level| {
                level
                    .iter()
                    .map(|n| if n.is_some() { 1 } else { 0 })
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0)
    }

    /// Align the nodes contained in the graph in layers.
    ///
    /// This performs the following steps:
    /// 1. Put each node in a layer, minimizing the height
    /// 2. Add padding to each level, so that each level has the same length
    /// 3. Reduce the number of crossings between to consecutive layers
    /// 4. Add spacing between the nodes
    fn align_nodes(&self) {
        if self.graph.node_count() == 0 {
            return;
        }

        // arrange nodes in levels,
        self.arrange_nodes_in_levels();

        // arrange vertically: moves nodes up as far as possible, by looking at successors
        for node in self.graph.node_identifiers().rev() {
            self.move_node_in_level(node, Direction::Outgoing)
        }
        //  arrange vertically: move nodes down as far as possible, by looking at predecessors
        for node in self.graph.node_identifiers() {
            self.move_node_in_level(node, Direction::Incoming)
        }

        // center levels
        let max_level_length = self
            .layers
            .borrow()
            .iter()
            .map(|level| level.len())
            .max()
            .unwrap();
        for level in self.layers.borrow_mut().iter_mut() {
            let mut padding = vec![None; (max_level_length - level.len()) / 2 + 1];
            padding.append(level);
            padding.append(&mut vec![None; (max_level_length - level.len()) / 2]);
            *level = padding;
        }

        // fill index_of_node
        for level in self.layers.borrow().iter() {
            for (index, node_opt) in level.iter().enumerate() {
                if let Some(node) = node_opt {
                    self.insert_index_of_node(*node, index);
                }
            }
        }

        for _ in 0..10 {
            for _ in 0..2 {
                let levels = self.layers.borrow().clone();
                for (level_index, level) in levels.into_iter().enumerate() {
                    for node in level.iter().skip(1).flatten() {
                        if let Some(left) = level[self.get_index_of_node(node).unwrap() - 1] {
                            self.reduce_crossings(*node, left, level_index)
                        }
                    }
                }
            }

            // swap with none neighbors
            for _ in 0..2 {
                let mut did_not_swap = true;
                let levels = self.layers.borrow().clone();
                for (level_index, level) in levels.iter().enumerate() {
                    for _ in 0..level.len() {
                        did_not_swap = true;
                        for node_opt in level.iter() {
                            let node = if let Some(node) = node_opt {
                                node
                            } else {
                                continue;
                            };
                            if !self.swap_with_none_neighbors(*node, level_index) {
                                did_not_swap = false;
                            }
                        }
                        if did_not_swap {
                            break;
                        }
                    }
                }
                if did_not_swap {
                    break;
                }
            }
        }

        #[cfg(feature = "debug")]
        self.print_layout(GraphPrintStyle::Char('#'));

        if self.global_tasks_in_first_row {
            for node in self.graph.node_identifiers() {
                let node_level = self.get_level_of_node(&node).unwrap();
                if node_level != 0
                    && self
                        .graph
                        .neighbors_directed(node, Direction::Incoming)
                        .count()
                        == 0
                {
                    self.layers.borrow_mut()[node_level]
                        .remove(self.get_index_of_node(&node).unwrap());
                    self.layers.borrow_mut()[0].push(Some(node));
                    self.insert_level_of_node(node, 0);
                }
            }
            for (node_index, node) in self.layers.borrow()[0].iter().enumerate() {
                if node.is_some() {
                    self.insert_index_of_node(node.unwrap(), node_index);
                }
            }
        }
    }

    #[inline(always)]
    fn arrange_nodes_in_levels(&self) {
        for node in toposort(&self.graph, None).unwrap() {
            let node_level = self
                .graph
                .neighbors_directed(node, Direction::Incoming)
                .filter_map(|predecessor| {
                    self.get_level_of_node(&predecessor).map(|level| level + 1)
                })
                .max()
                .unwrap_or(0);
            self.insert_level_of_node(node, node_level);
            self.add_node_to_level(node, node_level);
        }
    }

    /// Arrange Nodes in level depending on the direction.
    /// If the direction is Direction::Outgoing, it will try to move the nodes up as far as possible
    /// otherwise it will try to move the nodes as far down as possible
    #[inline(always)]
    fn move_node_in_level(&self, node: NodeIndex, direction: Direction) {
        let neighbor_levels = self
            .graph
            .neighbors_directed(node, direction)
            .filter_map(|neighbor| self.get_level_of_node(&neighbor));
        let new_node_level = match direction {
            Direction::Outgoing => neighbor_levels
                .min()
                .unwrap_or(self.get_nums_of_level())
                .saturating_sub(1), // move up
            Direction::Incoming => neighbor_levels.max().map(|level| level + 1).unwrap_or(0), // move down
        };

        let current_node_level = self.get_level_of_node(&node).unwrap();
        if current_node_level == new_node_level {
            return;
        }

        // remove the node from the old level, if it was already inserted before
        self.layers.borrow_mut()[current_node_level].retain(|other_node| &Some(node) != other_node);
        self.add_node_to_level(node, new_node_level);
        self.insert_level_of_node(node, new_node_level);
    }

    fn add_node_to_level(&self, node: NodeIndex, node_level: usize) {
        if let Some(level) = self.layers.borrow_mut().get_mut(node_level) {
            level.push(Some(node));
            return;
        }
        self.layers.borrow_mut().push(vec![Some(node)]);
    }

    fn reduce_crossings(&self, node: NodeIndex, left: NodeIndex, level_index: usize) {
        let get_direct_successors = |node| {
            self.graph
                .neighbors_directed(node, Direction::Outgoing)
                .filter(|n| self.get_level_of_node(n).unwrap().abs_diff(level_index) < 2)
                .collect::<Vec<_>>()
        };

        let successors = get_direct_successors(node);
        let left_successors = get_direct_successors(left);
        let mut cross_count = 0;
        let mut cross_count_swap = 0;
        for successor in successors {
            cross_count += left_successors
                .iter()
                .filter(|l_s| self.get_index_of_node(l_s) > self.get_index_of_node(&successor))
                .count();
            cross_count_swap += left_successors
                .iter()
                .filter(|l_s| self.get_index_of_node(l_s) < self.get_index_of_node(&successor))
                .count();
        }
        if cross_count_swap < cross_count {
            let level = &mut self.layers.borrow_mut()[level_index];
            let node_index = self.get_index_of_node(&node).unwrap();
            let left_index = self.get_index_of_node(&left).unwrap();
            level[node_index] = Some(left);
            level[left_index] = Some(node);

            self.insert_index_of_node(left, node_index);
            self.insert_index_of_node(node, left_index);
        }
    }

    fn swap_with_none_neighbors(&self, node: NodeIndex, level_index: usize) -> bool {
        let node_index = self.layers.borrow()[level_index]
            .iter()
            .position(|n| n == &Some(node))
            .unwrap();
        assert_ne!(node_index, 0);
        let left = if node_index == 0 {
            None
        } else {
            self.layers.borrow()[level_index][node_index - 1]
        };
        let right = if node_index >= self.layers.borrow()[level_index].len() - 1 {
            None
        } else {
            self.layers.borrow()[level_index][node_index + 1]
        };

        if left.is_some() && right.is_some() {
            return true;
        }

        let neighbor_indices: Vec<f64> = self
            .graph
            .neighbors_undirected(node)
            .filter(|neighbor| level_index.abs_diff(self.get_level_of_node(neighbor).unwrap()) < 2)
            .map(|neighbor| self.get_index_of_node(&neighbor).unwrap() as f64)
            .collect();

        if neighbor_indices.is_empty() {
            return true;
        }

        let mean_neighbor_index =
            neighbor_indices.iter().sum::<f64>() / neighbor_indices.len() as f64;

        // swap nodes for being closer to mean_neighbor_index
        let swap_index = if mean_neighbor_index < node_index as f64 - 0.5 && left.is_none() {
            node_index - 1
        } else if mean_neighbor_index > node_index as f64 + 0.5 && right.is_none() {
            node_index + 1
        } else {
            return true;
        };

        let level = &mut self.layers.borrow_mut()[level_index];
        level[node_index] = None;

        if swap_index > level.len() {
            level.push(Some(node));
        } else {
            level[swap_index] = Some(node);
        }

        self.insert_index_of_node(node, swap_index);

        false
    }

    /// Prints the graph to the console.
    ///
    /// Parameters:
    /// - style: GraphPrintStyle, the style in which the nodes of the graph are displayed.
    /// Can be either a specific char or the id of a node.
    #[cfg(feature = "debug")]
    fn print_layout(&self, style: GraphPrintStyle) {
        for l in self.layers.borrow().iter() {
            for n in l {
                if let Some(n) = n {
                    match &style {
                        GraphPrintStyle::Node => print!("{:>2?}, ", n.index()),
                        GraphPrintStyle::Char(c) => print!("{:}", c),
                    }
                } else {
                    print!(" ");
                }
            }
            println!("");
        }
    }
}

/// Specifies in which style a graph can be printed.
/// Variants are a user specified char or the id of a node.
#[cfg(feature = "debug")]
enum GraphPrintStyle {
    Node,
    Char(char),
}

#[cfg(test)]
mod tests {
    use super::GraphLayout;
    use petgraph::stable_graph::NodeIndex;

    #[test]
    fn test_into_weakly_connected_components_two_single_nodes() {
        let mut g = petgraph::stable_graph::StableDiGraph::<(), ()>::new();
        g.add_node(());
        g.add_node(());
        assert_eq!(GraphLayout::into_weakly_connected_components(g).len(), 2);
    }

    #[test]
    fn test_into_weakly_connected_compontents_two_components_correct_number_of_components() {
        let mut g = petgraph::stable_graph::StableDiGraph::<(), ()>::new();
        for _ in 0..4 {
            g.add_node(());
        }
        g.add_edge(NodeIndex::from(1), NodeIndex::from(0), ());
        g.add_edge(NodeIndex::from(2), NodeIndex::from(0), ());
        assert_eq!(GraphLayout::into_weakly_connected_components(g).len(), 2);
    }

    #[test]
    fn into_weakly_connected_components_two_components_correct_nodes_and_edges() {
        let g = petgraph::stable_graph::StableDiGraph::<(), ()>::from_edges([
            (0, 1),
            (1, 2),
            (3, 2),
            (4, 5),
            (4, 6),
        ]);
        let sgs = GraphLayout::into_weakly_connected_components(g);
        assert_eq!(sgs.len(), 2);
        assert!(sgs[0].contains_edge(0.into(), 1.into()));
        assert!(sgs[0].contains_edge(1.into(), 2.into()));
        assert!(sgs[0].contains_edge(3.into(), 2.into()));
        assert!(sgs[1].contains_edge(4.into(), 5.into()));
        assert!(sgs[1].contains_edge(4.into(), 6.into()));
    }
}
