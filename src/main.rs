
use std::collections::{HashMap, HashSet, BTreeMap};
use std::time;
use petgraph::Direction;
use petgraph::algo::{toposort};
use petgraph::stable_graph::{StableDiGraph};
use petgraph::graph::{DefaultIx, DiGraph, NodeIndex};
use petgraph::visit::{IntoNeighborsDirected, IntoNodeIdentifiers};

use graph_generator::{GraphLayout, RandomLayout};
use time::Instant;

static LAYOUT_1000: [(u32, u32); 1000] = [(0, 1), (1, 396), (396, 344), (1, 127), (344, 283), (344, 480), (1, 947), (1, 236), (344, 284), (396, 28), (1, 744), (1, 133), (283, 36), (1, 614), (396, 847), (847, 781), (283, 621), (744, 712), (344, 832), (28, 714), (28, 98), (744, 953), (744, 76), (396, 663), (284, 398), (396, 997), (344, 666), (344, 779), (396, 598), (1, 867), (947, 644), (779, 200), (779, 927), (744, 866), (344, 405), (344, 459), (947, 943), (283, 24), (1, 126), (283, 804), (133, 735), (927, 178), (1, 61), (0, 340), (344, 747), (779, 305), (61, 910), (396, 74), (127, 623), (1, 814), (1, 786), (623, 423), (340, 768), (28, 44), (598, 980), (283, 821), (98, 755), (910, 786), (781, 677), (344, 789), (396, 700), (396, 266), (28, 131), (283, 126), (396, 782), (947, 29), (340, 58), (396, 14), (997, 588), (283, 849), (340, 732), (283, 760), (480, 906), (1, 250), (804, 158), (396, 267), (250, 880), (735, 338), (1, 384), (867, 754), (131, 137), (947, 513), (623, 811), (947, 743), (947, 442), (1, 696), (700, 882), (14, 621), (267, 882), (781, 780), (781, 529), (781, 727), (305, 739), (779, 891), (910, 280), (396, 537), (283, 435), (744, 151), (714, 690), (712, 913), (744, 237), (779, 365), (947, 406), (910, 727), (779, 505), (344, 289), (847, 545), (913, 445), (910, 983), (396, 3), (405, 928), (44, 765), (765, 771), (61, 578), (396, 151), (1, 42), (623, 703), (284, 920), (1, 104), (714, 529), (58, 724), (779, 287), (236, 364), (744, 335), (598, 288), (76, 394), (714, 624), (910, 971), (396, 226), (396, 882), (821, 211), (700, 832), (880, 94), (28, 751), (744, 221), (284, 371), (712, 176), (545, 921), (396, 957), (405, 296), (340, 627), (910, 946), (283, 573), (779, 139), (953, 655), (910, 386), (396, 331), (250, 435), (910, 441), (28, 840), (953, 337), (131, 445), (768, 593), (344, 315), (768, 788), (910, 447), (396, 642), (405, 527), (158, 538), (396, 741), (305, 641), (804, 591), (267, 56), (739, 647), (236, 840), (744, 390), (847, 727), (744, 610), (315, 833), (305, 793), (847, 196), (755, 824), (712, 313), (910, 554), (953, 687), (283, 993), (1, 36), (687, 998), (983, 916), (284, 921), (139, 86), (781, 319), (751, 493), (554, 17), (296, 485), (1, 796), (537, 441), (283, 572), (284, 186), (754, 244), (283, 830), (847, 461), (847, 959), (344, 513), (14, 77), (28, 459), (751, 682), (739, 479), (386, 351), (771, 601), (283, 861), (1, 834), (14, 282), (910, 128), (0, 727), (305, 77), (880, 125), (610, 67), (610, 454), (14, 854), (696, 407), (396, 833), (910, 740), (405, 2), (340, 259), (861, 780), (768, 810), (700, 959), (712, 729), (386, 312), (847, 414), (186, 558), (340, 759), (916, 154), (296, 607), (910, 174), (17, 110), (396, 504), (821, 461), (396, 524), (847, 717), (1, 221), (283, 966), (916, 633), (768, 298), (682, 399), (744, 966), (927, 86), (641, 887), (283, 257), (830, 732), (743, 453), (364, 226), (700, 302), (700, 599), (283, 761), (714, 233), (289, 85), (1, 73), (396, 499), (910, 166), (315, 562), (302, 369), (998, 277), (779, 459), (947, 443), (642, 279), (396, 19), (573, 295), (910, 915), (847, 978), (406, 437), (28, 553), (480, 96), (139, 952), (283, 304), (405, 444), (279, 37), (174, 220), (910, 314), (642, 962), (880, 803), (0, 891), (58, 753), (803, 200), (755, 764), (96, 970), (315, 904), (331, 815), (14, 12), (666, 670), (61, 585), (28, 740), (28, 784), (771, 278), (610, 81), (14, 476), (1, 738), (396, 299), (700, 397), (302, 580), (396, 839), (834, 395), (76, 89), (804, 824), (781, 69), (739, 400), (295, 36), (735, 25), (1, 85), (81, 13), (14, 410), (283, 434), (927, 827), (690, 449), (14, 962), (1, 119), (666, 140), (454, 29), (277, 78), (140, 881), (573, 939), (396, 962), (623, 372), (283, 381), (744, 426), (396, 493), (1, 985), (131, 907), (953, 341), (847, 583), (499, 982), (910, 82), (1, 39), (947, 964), (946, 291), (624, 486), (1, 899), (1, 38), (847, 976), (751, 436), (712, 235), (947, 733), (947, 620), (821, 442), (396, 416), (910, 380), (572, 212), (76, 111), (396, 376), (396, 891), (405, 148), (396, 675), (866, 311), (593, 492), (302, 606), (1, 392), (910, 839), (779, 584), (700, 504), (386, 679), (779, 413), (847, 632), (910, 24), (744, 672), (623, 607), (847, 337), (666, 787), (700, 949), (627, 813), (744, 510), (1, 448), (1, 346), (250, 116), (127, 602), (17, 73), (434, 216), (344, 782), (397, 835), (781, 292), (744, 756), (781, 106), (259, 221), (781, 940), (372, 290), (1, 6), (744, 362), (283, 270), (362, 571), (804, 760), (340, 753), (314, 253), (738, 206), (296, 970), (866, 544), (396, 661), (319, 728), (344, 546), (712, 963), (283, 563), (396, 906), (804, 285), (283, 725), (284, 354), (283, 980), (738, 225), (341, 870), (335, 615), (340, 586), (346, 485), (927, 188), (747, 39), (305, 178), (947, 994), (1, 122), (104, 897), (344, 420), (344, 381), (284, 816), (744, 318), (953, 345), (28, 836), (623, 807), (340, 790), (96, 615), (405, 200), (74, 237), (344, 424), (1, 676), (396, 760), (744, 892), (405, 325), (870, 19), (76, 486), (104, 962), (910, 300), (344, 874), (887, 533), (396, 255), (259, 389), (127, 618), (480, 261), (315, 389), (768, 257), (546, 663), (344, 16), (174, 62), (916, 681), (583, 226), (1, 883), (768, 822), (861, 733), (1, 600), (712, 310), (700, 15), (1, 69), (295, 986), (675, 522), (803, 926), (480, 576), (296, 197), (545, 733), (744, 833), (1, 147), (583, 191), (724, 263), (28, 356), (847, 673), (283, 648), (947, 330), (1, 520), (396, 161), (1, 54), (454, 275), (346, 759), (787, 553), (372, 699), (449, 710), (947, 271), (480, 908), (61, 490), (983, 188), (971, 896), (947, 562), (127, 352), (910, 621), (700, 689), (396, 165), (396, 634), (813, 829), (296, 246), (396, 881), (13, 198), (690, 559), (910, 469), (1, 552), (125, 288), (0, 185), (648, 381), (480, 366), (703, 93), (396, 773), (751, 51), (449, 48), (283, 276), (263, 949), (897, 493), (926, 918), (267, 748), (724, 285), (803, 196), (283, 41), (344, 807), (661, 217), (283, 695), (236, 656), (910, 691), (384, 490), (292, 379), (714, 820), (789, 570), (712, 586), (712, 18), (396, 155), (700, 815), (405, 694), (880, 555), (1, 136), (480, 660), (997, 864), (279, 976), (623, 523), (803, 607), (755, 49), (874, 175), (127, 198), (93, 192), (744, 100), (346, 134), (279, 15), (864, 43), (396, 455), (703, 439), (781, 419), (738, 483), (864, 811), (632, 958), (396, 103), (946, 775), (176, 863), (376, 281), (1, 510), (910, 935), (675, 837), (836, 181), (28, 400), (847, 595), (910, 981), (804, 902), (284, 895), (346, 230), (642, 442), (803, 237), (67, 230), (492, 450), (1, 123), (997, 126), (344, 766), (76, 728), (1, 568), (655, 461), (744, 208), (998, 124), (712, 826), (449, 337), (266, 972), (128, 199), (396, 223), (416, 586), (318, 851), (1, 547), (724, 902), (279, 387), (94, 769), (1, 825), (887, 392), (283, 291), (344, 141), (197, 736), (449, 168), (910, 604), (1, 829), (714, 38), (148, 616), (298, 857), (344, 856), (315, 677), (287, 935), (28, 460), (524, 256), (492, 231), (371, 392), (1, 553), (563, 831), (687, 640), (755, 848), (744, 300), (724, 443), (398, 450), (897, 873), (724, 918), (787, 590), (191, 308), (804, 869), (896, 39), (67, 713), (14, 517), (946, 904), (555, 51), (67, 518), (263, 75), (449, 618), (835, 180), (946, 336), (854, 763), (283, 22), (1, 257), (604, 85), (537, 859), (344, 798), (100, 657), (907, 212), (398, 169), (58, 600), (279, 29), (74, 579), (781, 908), (315, 288), (670, 697), (283, 230), (847, 83), (283, 520), (284, 228), (754, 829), (983, 445), (947, 872), (283, 101), (48, 766), (666, 628), (67, 567), (284, 437), (54, 60), (136, 673), (405, 501), (910, 828), (869, 957), (910, 689), (524, 175), (17, 912), (305, 221), (916, 634), (751, 679), (449, 25), (744, 152), (483, 919), (139, 701), (744, 223), (744, 533), (283, 763), (340, 859), (284, 849), (714, 36), (259, 663), (803, 193), (870, 790), (296, 844), (910, 793), (407, 245), (714, 683), (803, 274), (604, 80), (279, 507), (354, 989), (1, 848), (642, 309), (738, 995), (1, 518), (524, 909), (449, 244), (724, 105), (971, 49), (947, 761), (61, 941), (744, 863), (420, 443), (610, 639), (953, 181), (344, 463), (821, 595), (499, 592), (997, 520), (43, 352), (847, 591), (165, 69), (813, 693), (131, 968), (687, 102), (755, 216), (874, 646), (610, 832), (287, 965), (1, 473), (315, 274), (947, 62), (910, 224), (12, 580), (896, 137), (319, 980), (851, 353), (1, 304), (13, 977), (28, 863), (648, 478), (283, 848), (601, 679), (1, 239), (191, 777), (910, 761), (910, 445), (1, 152), (947, 514), (724, 900), (847, 452), (449, 18), (104, 423), (880, 422), (744, 976), (623, 375), (713, 656), (880, 660), (473, 858), (1, 368), (279, 875), (947, 117), (1, 961), (396, 808), (576, 517), (364, 221), (768, 716), (601, 807), (266, 493), (384, 990), (953, 921), (396, 262), (524, 938), (104, 591), (1, 536), (953, 543), (449, 151), (915, 963), (744, 412), (953, 25), (724, 736), (601, 506), (284, 855), (305, 243), (666, 948), (344, 77), (133, 758), (295, 817), (927, 975), (779, 902), (781, 475), (41, 254), (76, 561), (344, 566), (958, 579), (405, 379), (916, 157), (947, 942), (779, 842), (284, 525), (48, 651), (873, 52), (787, 394), (744, 783), (492, 245), (1, 390), (82, 615), (755, 972), (781, 456), (856, 882), (279, 720), (714, 662), (928, 681), (744, 233), (131, 528), (947, 485), (17, 791), (687, 203), (384, 106), (948, 327), (808, 544), (28, 306), (601, 427), (953, 403), (396, 53), (396, 264), (473, 785), (279, 709), (927, 336), (953, 590), (17, 89), (804, 562), (657, 5), (947, 429), (1, 748), (279, 411), (396, 614), (571, 2), (861, 940), (396, 438), (28, 359), (396, 417), (478, 707), (744, 857), (344, 89), (751, 519), (104, 439), (655, 825), (700, 297), (738, 718), (208, 423), (946, 502), (754, 920), (720, 154), (119, 469), (340, 484), (744, 235), (283, 996), (642, 966), (434, 455), (0, 312), (744, 933), (601, 805), (714, 356), (449, 527), (670, 452), (751, 36), (0, 135), (344, 827), (396, 435), (803, 99), (687, 608), (266, 352), (703, 853), (537, 204), (545, 147), (666, 811), (61, 370), (947, 843), (344, 684), (295, 570), (396, 60), (897, 872), (1, 853), (298, 391), (804, 895), (1, 168), (111, 350), (997, 658), (787, 496), (627, 986), (1, 957), (847, 257), (186, 265), (76, 505), (283, 149), (136, 759), (197, 486), (284, 453), (942, 811), (821, 945), (305, 581), (283, 936), (568, 520), (427, 734), (28, 597), (700, 711), (754, 611), (608, 297), (910, 164), (744, 679), (910, 375), (315, 51), (844, 819), (449, 452), (76, 103), (948, 256), (284, 490), (627, 256), (779, 195), (340, 902), (821, 145), (946, 774), (743, 275), (938, 168), (305, 722), (661, 328), (305, 85), (427, 36), (283, 184), (429, 27), (289, 16), (658, 860), (366, 717), (449, 639), (779, 699), (993, 4), (76, 226), (781, 408), (478, 168), (554, 621), (910, 92), (847, 688), (514, 826), (255, 793), (364, 930), (571, 848), (880, 827), (724, 616), (124, 656), (478, 975), (279, 764), (480, 665), (281, 57), (61, 805), (910, 794), (344, 343), (480, 819), (14, 859), (683, 39), (768, 435), (847, 363), (58, 968), (803, 698), (478, 144), (661, 225), (910, 207), (779, 654), (396, 249), (28, 31), (396, 607), (417, 937), (779, 746), (927, 137), (136, 621), (17, 535), (287, 387), (305, 162), (174, 203), (136, 173), (910, 431), (191, 56), (847, 380), (478, 69), (856, 628), (927, 381), (1, 275), (302, 179), (948, 485), (283, 489), (279, 848), (128, 811), (127, 941), (861, 487), (751, 348), (124, 57), (302, 173), (96, 860), (744, 988), (687, 162), (867, 518), (396, 883), (755, 872), (910, 476), (405, 388), (571, 742), (916, 979), (744, 65), (139, 173), (58, 859), (148, 767), (915, 497), (405, 899), (751, 413), (910, 40), (661, 451), (754, 784), (751, 668), (388, 852), (149, 933), (1, 530), (396, 507), (794, 274), (28, 49), (302, 988), (48, 832), (396, 709), (821, 903)];

fn main() {
    // create graph
    let layout = GraphLayout::new_from_num_nodes(22, 2);
    // let layout = RandomLayout::new(1000);
    let _ = graph_generator::write_to_file("22_2", &layout.build_edges());
    let edges = layout.build_edges().into_iter().map(|(n, s): (usize, usize)| (n as u32, s as u32)).collect::<Vec<(u32, u32)>>();
    // let g = StableDiGraph::<i32, i32>::from_edges(
    //     &[(1, 2), (0, 1), (0, 6), (6, 7), (1, 7), (7, 8), (7, 9), (7, 10)]
    // );
    println!("start");
    let start = Instant::now();
    let g = StableDiGraph::<i32, i32>::from_edges(&edges);
    let layout: BTreeMap<_, _> = graph_layout(g).unwrap().0[0].clone().into_iter().collect();
    let end = start.elapsed().as_micros();
    println!("{} us.\n {:?}", end, layout);
}

// node index, (x, y)
type Layout = HashMap<usize, (isize, isize)>;

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

fn handle_two_or_less_nodes_graph(
    graph: StableDiGraph<i32, i32>,
    node_separation: isize,
    width_list: &mut Vec<usize>,
    height_list: &mut Vec<usize>,
    layout_list: &mut Vec<Layout>)
{
    let mut layout_tmp = Layout::new();
    for (node_index, node) in graph.node_indices().enumerate() {
        let x = node_separation;
        let y = -(node_index as isize) * node_separation;
        layout_tmp.insert(node.index(), (x, y));
    }
    width_list.push(1);
    height_list.push(graph.node_count());
    layout_list.push(layout_tmp);
}

fn create_nodes_in_level(graph: &StableDiGraph<i32, i32>, level_of_node: &mut HashMap<NodeIndex, usize>) -> Vec<Vec<Option<NodeIndex>>> {
    let mut nodes_in_level: Vec<Vec<Option<NodeIndex>>> = Vec::new();
    for node in toposort(graph, None).unwrap() {
        let node_level = graph.neighbors_directed(node, Direction::Incoming)
            .filter_map(|predecessor| level_of_node.get(&predecessor))
            .max()
            .unwrap_or(&0)
            + 1;

        level_of_node.insert(node, node_level);
        add_node_to_level(node, node_level, &mut nodes_in_level);
    }

    nodes_in_level
}

/// Arrange Nodes in level depending on the direction.
/// If the direction is Direction::Outgoing, it will try to move the nodes up as far as possible
/// otherwise it will try to move the nodes as far down as possible
// TODO: In the original version, when direction is outgoing, the nodes should be iterated over in reverse order
// TODO: This doesn't seem to make a difference in the end result though
fn move_nodes_in_level(
    graph: &StableDiGraph<i32, i32>,
    nodes_in_level: &mut Vec<Vec<Option<NodeIndex>>>,
    level_of_node: &mut HashMap<NodeIndex, usize>,
    direction: Direction
) {
    for node in graph.node_indices() {
        let neighbor_levels = graph.neighbors_directed(node, direction).filter_map(|neighbor| level_of_node.get(&neighbor));
        let new_node_level = match direction {
            Direction::Incoming => *neighbor_levels.max().unwrap_or(&0) + 1,
            Direction::Outgoing => neighbor_levels.min().unwrap_or(&nodes_in_level.len()).checked_sub(1).unwrap_or(0)
        };

        let current_node_level = *level_of_node.get(&node).unwrap();
        if current_node_level == new_node_level { continue }

        // remove the node from the old level, if it was already inserted before
        nodes_in_level[current_node_level].retain(|other_node| &Some(node) != other_node);
        add_node_to_level(node, new_node_level, nodes_in_level);
        level_of_node.insert(node, new_node_level);
    }
}

fn add_node_to_level(node: NodeIndex, node_level: usize, nodes_in_level: &mut Vec<Vec<Option<NodeIndex>>>) {
    if let Some(level) = nodes_in_level.get_mut(node_level) {
        level.push(Some(node));
    } else {
        while nodes_in_level.len() <= node_level {
            nodes_in_level.push(vec![]);
        }
        nodes_in_level[node_level].push(Some(node));
    }
}

fn graph_layout(graph: StableDiGraph<i32, i32>) -> Option<(Vec<Layout>, Vec<usize>, Vec<usize>)> {
    let node_size: isize = 40;
    let node_separation = 4 * node_size;
    let global_tasks_in_first_row = false;

    if graph.node_count() == 0 {
        return None;
    }

    let graph_list = into_weakly_connected_components(graph);

    let mut layout_list = Vec::<Layout>::new();
    let mut height_list = Vec::new();
    let mut width_list = Vec::new();

    for g in graph_list {
        let mut layout_tmp = Layout::new();

        // case for one or two nodes
        if g.node_count() <= 2 {
            handle_two_or_less_nodes_graph(
                g,
                node_separation,
                &mut width_list,
                &mut height_list,
                &mut layout_list
            );
            continue
        }

        let mut level_of_node = HashMap::<NodeIndex, usize>::new();  // level for each node
        let mut index_of_node = HashMap::<NodeIndex, usize>::new();  // index for each node
        // arrange nodes in levels,
        let mut nodes_in_level = create_nodes_in_level(&g, &mut level_of_node);

        // arrange vertically: moves nodes up as far as possible, by looking at successors
        move_nodes_in_level(
            &g,
            &mut nodes_in_level,
            &mut level_of_node,
            Direction::Outgoing
        );

        //  arrange vertically: move nodes down as far as possible, by looking at predecessors
        move_nodes_in_level(
            &g,
            &mut nodes_in_level,
            &mut level_of_node,
            Direction::Incoming
        );

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
                }
            }
        }


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

            // swap with none neighbors
            for _ in 0..2 {
                let mut did_not_swap = true;
                for (level_index, level) in nodes_in_level.clone().iter().enumerate() {
                    let mut swap_count = 0;
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
                            for neighbor in g.neighbors_undirected(node) {
                                if level_index.abs_diff(*level_of_node.get(&neighbor).unwrap()) < 2 {
                                    mean_neighbor_index += *index_of_node.get(&neighbor).unwrap() as f64;
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
                }
                if did_not_swap {
                    break;
                }
            }
        }
        print_layout(&nodes_in_level, PrintStyle::Char('#'));

        // println!("swap all: {} us", start.elapsed().as_micros());

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

        // println!("{}", nodes_in_level.iter().map(|l| l.len()).sum::<usize>());

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

        height_list.push(nodes_in_level.len());
        layout_list.push(layout_tmp);
    }


    return Some((layout_list, width_list, height_list))
}

enum PrintStyle {
    Node,
    Char(char),
}

fn print_layout(layout: &[Vec<Option<NodeIndex>>], style: PrintStyle) {
    for l in layout {
        for n in l {
            if let Some(n) = n {
                match &style {
                    PrintStyle::Node => print!("{:>2?}, ", n.index()),
                    PrintStyle::Char(c) => print!("{:}", c),
                }

            } else {
                print!(" ");
            }
        }
        println!("");
    }
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

