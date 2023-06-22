use std::cmp::min;
use std::collections::{HashMap, HashSet, BTreeMap};
use std::time;
use petgraph::{Directed, Direction, Graph};
use petgraph::algo::{connected_components, tarjan_scc, toposort};
use petgraph::stable_graph::{StableDiGraph, StableGraph};
use petgraph::graph::{DefaultIx, DiGraph, NodeIndex};
use petgraph::unionfind::UnionFind;
use petgraph::visit::{IntoEdgeReferences, IntoNeighborsDirected, IntoNodeIdentifiers, NodeCompactIndexable};

use graph_generator::GraphLayout;
use time::Instant;

fn main() {
    // create graph
    let layout = GraphLayout::new_from_num_nodes(50, 2);
    let _ = layout.build_edges_and_write_to_file("1000_2");
    let edges = layout.build_edges().into_iter().map(|(n, s): (usize, usize)| (n as u32, s as u32)).collect::<Vec<(u32, u32)>>();
    // let g = StableDiGraph::<i32, i32>::from_edges(
    //     &[(1, 2), (0, 1), (0, 6), (6, 7), (1, 7), (7, 8), (7, 9), (7, 10)]
    // );
    println!("start");
    let start = Instant::now();
    let g = StableDiGraph::<i32, i32>::from_edges(&edges);
    let layout: BTreeMap<_, _> = graph_layout(g).unwrap().0[0].clone().into_iter().collect();
    let end = start.elapsed().as_micros();
    println!("{} us.\n{:?}", end, layout);
}

fn graph_layout(graph: StableDiGraph<i32, i32>) -> Option<(Vec<HashMap<usize, (isize, isize)>>, Vec<usize>, Vec<usize>)> {
    let node_size: isize = 40;
    let node_separation = 4 * node_size;
    let global_tasks_in_first_row = false;

    if graph.node_count() == 0 {
        return None;
    }

    let graph_list = into_weakly_connected_components(graph);
    let number_of_independent_graphs = graph_list.len();

    let mut layout_list = Vec::<HashMap<usize, (isize, isize)>>::new();
    let mut height_list = vec![0; number_of_independent_graphs];
    let mut width_list = vec![0; number_of_independent_graphs];

    for (layout_i, g) in graph_list.into_iter().enumerate() {
        let mut layout_tmp = HashMap::<usize, (isize, isize)>::new();

        // case for one or two nodes
        if g.node_count() <= 2 {
            // NOTE: do these need to be sorted?
            for (node_i, node) in g.node_indices().enumerate() {
                let x = node_separation;
                let y = -(node_i as isize) * node_separation;
                layout_tmp.insert(node.index(), (x, y));
            }
            width_list[layout_i] = 1;
            height_list[layout_i] = g.node_count();
            layout_list.push(layout_tmp);
            continue;
        }

        let mut level_of_node = HashMap::<NodeIndex, usize>::new();  // level for each node
        let mut index_of_node = HashMap::<NodeIndex, usize>::new();  // index for each node
        let mut nodes_in_level: Vec<Vec<Option<NodeIndex>>> = vec![vec![]];  // nodes in each level
        let mut number_of_levels = 1;  // total number of levels
        let mut neighbours = HashMap::new();   // list of neighbours with more than one dependency for each node
        let mut nodes = Vec::new();  // list of nodes with more than one dependency

        // fill predecessors_of_node, successors_of_node etc.
        for node in g.node_indices() {
            neighbours.insert(node, Vec::new());
            for neighbor in g.neighbors_undirected(node) {
                let entry = neighbours.entry(node).or_insert(Vec::new());
                entry.push(neighbor);
            }

            nodes.push(node);
        }


        // arrange all nodes of in levels,
        for node in toposort(&g, None).unwrap() {
            // find maximum level of predecessors
            let mut max_predecessor_level: usize = 0;
            for predecessor in g.neighbors_directed(node, Direction::Incoming) {
                max_predecessor_level = std::cmp::max(
                    max_predecessor_level,
                    *level_of_node.get(&predecessor).unwrap_or(&0)
                );
            }
            // put node one level below
            let node_level = max_predecessor_level + 1;
            // node_level is 0 based index
            if node_level >= number_of_levels {
                number_of_levels += 1;
                nodes_in_level.push(Vec::new());
            }
            nodes_in_level.get_mut(node_level).unwrap().push(Some(node));
            level_of_node.insert(node, node_level);
        }

        // arrange vertically: moves nodes up as far as possible
        for node in g.node_indices() {
            // find minimum level of successors
            let min_successor_level = *std::cmp::min(
                g
                    .neighbors_directed(node, Direction::Outgoing)
                    .map(|node| level_of_node.get(&node))
                    .flatten()
                    .min()
                    .unwrap_or(&usize::MAX),
                &number_of_levels);

            if level_of_node[&node] == min_successor_level - 1 {
                continue;
            }

            // put node one level above successor
            let node_level = min_successor_level - 1;
            nodes_in_level[*level_of_node.get(&node).unwrap()].retain(|other_node| &Some(node) != other_node); // remove the node
            nodes_in_level[node_level].push(Some(node));
            level_of_node.entry(node).and_modify(|entry| *entry = node_level);
        }
        
        //  arrange vertically: move nodes down as far as possible
        for node in g.node_indices() {
            let max_predecessor_level = *std::cmp::max(
                g
                    .neighbors_directed(node, Direction::Incoming)
                    .filter(|neighbor| neighbours.get(&node)
                            .unwrap()
                            .contains(neighbor))
                    .map(|neighbor| level_of_node.get(&neighbor))
                    .flatten()
                    .max(),
                Some(&0))
                .unwrap();

            if level_of_node[&node] == max_predecessor_level + 1 {
                continue;
            }

            // put node one level below
            let node_level = max_predecessor_level + 1;
            if node_level >= number_of_levels {
                number_of_levels += 1;
                nodes_in_level.push(Vec::new());
            }
            // remove the node
            nodes_in_level[*level_of_node.get(&node).unwrap()].retain(|other_node| &Some(node) != other_node);
            nodes_in_level[node_level].push(Some(node));
            level_of_node.entry(node).and_modify(|entry| *entry = node_level);
        }
        
        // center levels
        let max_level_length = nodes_in_level.iter().map(|level| level.len()).max().unwrap();
        for level in nodes_in_level.iter_mut() {
            let level_length = level.len();
            let mut padding = vec![None; (max_level_length - level_length) / 2 + 1];
            padding.append(level);
            padding.append(&mut vec![None; (max_level_length - level_length) / 2]);
            *level = padding;
        }

        // fill index_of_node
        for level in &nodes_in_level {
            for (index, node_opt) in level.iter().enumerate() {
                if let Some(node) = node_opt {
                    index_of_node.insert(*node, index);
                    // index_of_node.entry(*node).and_modify(|e| *e = index);
                }
            }
        }

        // print_layout(&nodes_in_level);

        let start = Instant::now();
            // let start_crossings = Instant::now();
        for _ in 0..10 {
            for _ in 0..2 {
                for (level_index, level) in nodes_in_level.clone().into_iter().enumerate() {
                    for node_opt in level.iter().skip(1) {
                        if node_opt.is_none() {
                            continue;
                        }
                        let node = node_opt.unwrap();
                        let left = if let Some(left) = level[*index_of_node.get(&node).unwrap() - 1] {
                            left
                        } else {
                            continue;
                        };

                        let successors: Vec<_> = g.neighbors_directed(node, Direction::Outgoing)
                            .filter(|n| level_of_node.get(n).unwrap() - level_index < 2)
                            .collect();
                        let left_successors: Vec<_> = g.neighbors_directed(left, Direction::Outgoing)
                            .filter(|n| level_of_node.get(n).unwrap() - level_index < 2)
                            .collect();
                        let mut cross_count = 0;
                        let mut cross_count_swap = 0;
                        for successor in successors {
                            cross_count += left_successors.iter()
                                .filter(|l_s| index_of_node.get(l_s) > index_of_node.get(&successor))
                                .count();
                            cross_count_swap += left_successors.iter()
                                .filter(|l_s| index_of_node.get(l_s) < index_of_node.get(&successor))
                                .count();
                        }
                        if cross_count_swap < cross_count {
                            let level = nodes_in_level.get_mut(level_index).unwrap();
                            let node_index = *index_of_node.get(&node).unwrap();
                            let left_index = *index_of_node.get(&left).unwrap();
                            level[node_index] = Some(left);
                            level[left_index] = Some(node);

                            index_of_node.insert(left, node_index);
                            index_of_node.insert(node, left_index);
                        }
                    }
                }
            }
            // println!("swap crossings: {}", start_crossings.elapsed().as_micros());

            // swap with none neighbors
            for _ in 0..2 {
                let mut did_not_swap = true;
                // print_layout(&nodes_in_level);
                for (level_index, level) in nodes_in_level.clone().iter().enumerate() {
                    let mut swap_count = 0;
                    let start_none = Instant::now();
                    for _ in 0..level.len() / 2 {
                        did_not_swap = true;
                        for node_opt in level.iter() {
                            let node = if let Some(node) = node_opt { *node } else { continue; };
                            let node_index = nodes_in_level[level_index].iter().position(|n| n == &Some(node)).unwrap();
                            let left = if node_index == 0 { None } else { nodes_in_level[level_index][node_index - 1] };
                            let right = if node_index == nodes_in_level[level_index].len() - 1 { None } else { nodes_in_level[level_index][node_index + 1] };

                            if left.is_some() && right.is_some() {
                                continue;
                            }

                            let mut mean_neighbor_index = 0.;
                            let mut count = 0.;
                            for neighbor in neighbours.get(&node).unwrap() {
                                if level_index.abs_diff(*level_of_node.get(neighbor).unwrap()) < 2 {
                                    mean_neighbor_index += *index_of_node.get(neighbor).unwrap() as f64;
                                    count += 1.;
                                }
                            }

                            if count == 0. {
                                continue;
                            }
                            mean_neighbor_index /= count;

                            // swap nodes for being closer to mean_neighbor_index
                            if mean_neighbor_index < node_index as f64 - 0.5 && left.is_none() {
                                swap_count += 1;
                                did_not_swap = false;
                                nodes_in_level[level_index][node_index] = None;
                                nodes_in_level[level_index][node_index - 1] = Some(node);
                                index_of_node.insert(node, node_index - 1);
                            } else if mean_neighbor_index > node_index as f64 + 0.5 && right.is_none() {
                                swap_count += 1;
                                did_not_swap = false;
                                let level = nodes_in_level.get_mut(level_index).unwrap();
                                level[node_index] = None;
                                if node_index + 1 >= level.len() {
                                    level.push(Some(node));
                                } else {
                                    level[node_index + 1] = Some(node);
                                }
                                index_of_node.insert(node, node_index + 1);
                            }
                        }
                        if did_not_swap {
                            break;
                        }
                    }
                    // println!("swap none: {} us\tlvl: {}\t swap_count: {}", start_none.elapsed().as_micros(), level_index, swap_count);
                }
                if did_not_swap {
                    break;
                }
            }
        }
        print_layout(&nodes_in_level);

        println!("swap all: {} us", start.elapsed().as_micros());

        if global_tasks_in_first_row {
            for node in g.node_identifiers() {
                let node_level = *level_of_node.get(&node).unwrap(); 
                if  node_level != 0 && g.neighbors_directed(node, Direction::Incoming).count() == 0 {
                    nodes_in_level[node_level].remove(*index_of_node.get(&node).unwrap());
                    nodes_in_level[0].push(Some(node));
                    level_of_node.insert(node, 0);
                }
            }
            for (node_index, node) in nodes_in_level[0].iter().enumerate() {
                if node.is_some() {
                    index_of_node.insert(node.unwrap(), node_index);
                }
            }
        }

        println!("{}", nodes_in_level.iter().map(|l| l.len()).sum::<usize>());

        // build layout
        let offset = if nodes_in_level[0].iter().all(|n| n.is_none()) { 1 } else { 0 };
        for (level_index, level) in nodes_in_level.iter().enumerate() {
            for (node_index, node_opt) in level.iter().enumerate() {
                let node = if let Some(node) = node_opt { *node } else { continue; };
                let x = node_index as isize * node_separation;
                let y = (-(level_index as isize) + offset) * node_separation;
                layout_tmp.insert(node.index(), (x, y));
            }
        }

        height_list[layout_i] = number_of_levels;
        layout_list.push(layout_tmp);
    }


    return Some((layout_list, width_list, height_list))
}

fn print_layout(layout: &[Vec<Option<NodeIndex>>]) {
    for l in layout {
        for n in l {
            if let Some(n) = n {
                print!("{:>2?}, ", n.index());
            } else {
                print!("  , ");
            }
        }
        println!("");
    }
}

fn into_weakly_connected_components(graph: StableDiGraph<i32, i32>) -> Vec<StableDiGraph<i32, i32>> {
    let mut visited = HashSet::<NodeIndex>::new();
    let sorted_identifiers = toposort(&graph, None).unwrap();
    let mut sub_graphs = Vec::new();

    // build each subgraph
    for identifier in sorted_identifiers {
        let mut subgraph_edges = vec![];
        let mut sources = vec![identifier];

        // since graph is sorted, we only need to look for successors
        while let Some(source) = sources.pop() {
            if !visited.insert(source) {
                continue;
            }
            let successors = graph.neighbors_directed(source, Direction::Outgoing);
            for successor in successors {
                subgraph_edges.push((source.index() as DefaultIx, successor.index() as DefaultIx)); // NOTE: will this work, if nodes contain actual data?
                sources.push(successor);
            }
        }
        if subgraph_edges.len() > 0 {
            sub_graphs.push(StableDiGraph::from_edges(subgraph_edges));
        }
    }

    return sub_graphs
}

// chatgpt generated code
use petgraph::visit::Bfs;

// Function to transform the graph into a layered layout
fn _layered(graph: &DiGraph<(), ()>) -> Vec<Vec<NodeIndex>> {
    let mut layers: Vec<Vec<NodeIndex>> = Vec::new();

    // Perform BFS traversal to determine the layers
    let mut bfs = Bfs::new(&graph, NodeIndex::new(0)); // Start traversal from a specific node (e.g., root)
    while let Some(node) = bfs.next(&graph) {
        let level = 0;//bfs.depth(&graph, node) as usize;

        // Ensure the level exists in the `layers` vector
        if level >= layers.len() {
            layers.push(Vec::new());
        }

        // Add the node to the corresponding layer
        layers[level].push(node);
    }

    layers
}

// Usage example
fn _main() {
    // Create a directed graph using the `petgraph` crate
    let mut graph = DiGraph::<(), ()>::new();

    // Add nodes and edges to the graph
    let node_a = graph.add_node(());
    let node_b = graph.add_node(());
    let node_c = graph.add_node(());
    let node_d = graph.add_node(());

    graph.add_edge(node_a, node_b, ());
    graph.add_edge(node_b, node_c, ());
    graph.add_edge(node_c, node_d, ());

    // Perform the layered layout transformation
    let layers = _layered(&graph);

    // Print the resulting layers
    for (level, nodes) in layers.iter().enumerate() {
        println!("Level {}: {:?}", level, nodes);
    }
}

