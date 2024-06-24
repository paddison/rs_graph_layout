"""
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
"""

import sys
sys.path.append('/Users/patrick/Programming/rust/projects/graph_layout_python/python/nx_venv/lib/python3.12/site-packages')

def graph_layout(edges, node_separation, global_tasks_in_first_row):
    import networkx as nx
    import itertools

    graph = nx.DiGraph()
    graph.add_edges_from(edges)

    if graph.number_of_nodes() == 0:
        return None

    graph_list = list(graph.subgraph(c) for c in nx.weakly_connected_components(graph))
    number_of_independent_graphs = len(graph_list)

    layout_list = []
    height_list = [0] * number_of_independent_graphs
    width_list = [0] * number_of_independent_graphs

#        total_cross_count = 0
#        total_edge_length = 0

    for layout_i, g in enumerate(graph_list):
        layout_tmp = {}

        # case for one or two nodes
        if g.number_of_nodes() <= 2:
            i_n = 0
            for n in sorted(g.nodes()):
                x = node_separation
                y = -i_n * node_separation
                layout_tmp[n] = (x, y)
                #layout_list[layout_i][n] = (x + layout_xmax, y)
                #layout_list[n] = (x + layout_xmax, y)
                i_n += 1
            #layout_xmax = layout_xmax + self.node_separation
            width_list[layout_i] = 1
            height_list[layout_i] = g.number_of_nodes()
            layout_list.append(layout_tmp)
            continue

        # set up data arrays
        level_of_node = {}  # level for each node
        index_of_node = {}  # index for each node
        nodes_in_level = [[]]  # nodes in each level
        number_of_levels = 1  # total number of levels
        single_dep_neighbours = {}  # list of neighbours with only one
                                    # dependency for each node
        multi_dep_neighbours = {}  # list of neighbours with more than one
                                   # dependency for each node
        single_dep_nodes = []  # list of nodes with exactly one dependency
        multi_dep_nodes = []  # list of nodes with more than one dependency

        # fill predecessors_of_node, successors_of_node etc.
        for n in g.nodes():
            single_dep_neighbours[n] = []
            multi_dep_neighbours[n] = []

            for x in itertools.chain(g.predecessors(n), g.successors(n)):
                if g.degree(x) == 1:
                    single_dep_neighbours[n].append(x)
                else:
                    multi_dep_neighbours[n].append(x)

            if g.degree(n) == 1:
                single_dep_nodes.append(n)
            elif g.degree(n) > 1:
                multi_dep_nodes.append(n)

        # create subgraph with multi dependency nodes only
        subgraph = g.subgraph(multi_dep_nodes)

        # arrange all nodes of subgraph in levels,
        for n in nx.topological_sort(subgraph):
            # find maximum level of predecessors
            maxprelev = 0
            for p in subgraph.predecessors(n):
                maxprelev = max(maxprelev, level_of_node[p])
            # put node one level below
            nlev = maxprelev + 1
            if nlev >= number_of_levels:  # nlev is a 0 based index
                number_of_levels += 1
                nodes_in_level.append([])
            nodes_in_level[nlev].append(n)
            level_of_node[n] = nlev

        # arrange vertically: move nodes up as far as possible
        for l in reversed(nodes_in_level):
            for n in l[:]:
                if subgraph.out_degree(n) == 0:
                    continue
                # find minimum level of successors
                minsuclev = number_of_levels
                for s in subgraph.successors(n):
                    minsuclev = min(minsuclev, level_of_node[s])
                if level_of_node[n] == minsuclev - 1:
                    continue
                # put node one level above successor
                nlev = minsuclev - 1
                nodes_in_level[level_of_node[n]].remove(n)
                nodes_in_level[nlev].append(n)
                level_of_node[n] = nlev

        # arrange vertically: move nodes down as far as possible
        for l in nodes_in_level:
            for n in l[:]:
                if subgraph.in_degree(n) == 0:
                    continue
               # find maximum level of predecessors
                maxprelev = 0
                for p in subgraph.predecessors(n):
                    if p in multi_dep_neighbours[n]:
                        maxprelev = max(maxprelev, level_of_node[p])
                if level_of_node[n] == maxprelev + 1:
                    continue
                # put node one level below
                nlev = maxprelev + 1
                if nlev >= number_of_levels:  # nlev is a 0 based index
                    number_of_levels += 1
                    nodes_in_level.append([])
                nodes_in_level[level_of_node[n]].remove(n)
                nodes_in_level[nlev].append(n)
                level_of_node[n] = nlev

        # center levels
        max_level_len = max([len(l) for l in nodes_in_level])
        for l in nodes_in_level:
            levlen = len(l)
            l[0:] = [None]*((max_level_len-levlen)//2+1) + l + \
                [None]*((max_level_len-levlen)//2)

        # fill index_of_node
        for l in nodes_in_level:
            for n in l:
                if n is not None:
                    index_of_node[n] = l.index(n)

        # swap nodes
        for i in range(1):  # 6
            l_i = 0  # list index
            for l in nodes_in_level:
                for n in l[1:]:
                    if n is None:
                        continue
                    left = l[index_of_node[n]-1]
                    suc = [x for x in subgraph.successors(n)
                           if level_of_node[x]-l_i < 2]
                    if left is None:
                        continue
                    left_suc = [x for x in subgraph.successors(left)
                                if level_of_node[x]-l_i < 2]

                    cross_count = 0
                    cross_count_swap = 0
                    for s in suc:
                        cross_count += \
                            len([x for x in left_suc
                                 if index_of_node[x] > index_of_node[s]])
                        cross_count_swap += \
                            len([x for x in left_suc
                                 if index_of_node[x] < index_of_node[s]])

                    # swap nodes if it results in less crossings
                    if cross_count_swap < cross_count:

                        l[index_of_node[n]] = left
                        l[index_of_node[left]] = n

                        index_tmp = index_of_node[left]
                        index_of_node[left] = index_of_node[n]
                        index_of_node[n] = index_tmp

                l_i += 1

        # swap nodes with None neighbours
        for i in range(10):  # 6
            break_flag = True
            l_i = 0  # list index
            for l in nodes_in_level:
                for kk in range(len(l)//2):
                    break_flag = True
                    for n in l[1:-2]:
                        if n is None:
                            continue
                        i_n = l.index(n)
                        left = l[i_n-1]
                        right = l[i_n+1]
                        suc = [x for x in subgraph.successors(n)
                               if level_of_node[x]-l_i < 2]
                        if left is not None and right is not None:
                            continue

                        mean_neighbour_index = 0  # nodes_tmp.index(n)
                        count = 0.0
                        for x in multi_dep_neighbours[n]:
                            if True:  # x in g.successors(n):
                                if abs(level_of_node[n] -
                                       level_of_node[x]) < 2:
                                    mean_neighbour_index += \
                                        nodes_in_level[
                                            level_of_node[x]].index(x)
                                    count += 1
                        if count == 0:
                            continue
                        mean_neighbour_index /= count

                        # swap nodes for being closer to
                        # mean_neighbour_index
                        if (mean_neighbour_index < i_n-.5 and
                                left is None):
                            break_flag = False
                            l[i_n] = None
                            l[i_n-1] = n
                            index_of_node[n] = i_n - 1
                        elif (mean_neighbour_index > i_n+.5 and
                                right is None):
                            break_flag = False
                            l[i_n] = None
                            l[i_n+1] = n
                            index_of_node[n] = i_n + 1

                    if break_flag is True:
                        break

                l_i += 1

            if break_flag is True:
                break

        # sort in single dependency nodes
        for l in nodes_in_level:
            for n in l[:]:
                if n is None or n in single_dep_nodes:
                    continue
                for p in g.predecessors(n):
                    if p in single_dep_nodes:
                        i_n = l.index(n)
                        if (len(nodes_in_level[level_of_node[n] - 1])
                                > i_n and
                                nodes_in_level[level_of_node[n] - 1][i_n]
                                is None):
                            nodes_in_level[level_of_node[n]-1][i_n] = p
                        else:
                            #print n,p,level_of_node[n], i_n
                            nodes_in_level[level_of_node[n]-1].insert(
                                i_n, p)
                            nodes_in_level[level_of_node[n]].insert(
                                i_n, None)
                            # if level_of_node[n] > 1:
                                # nodes_in_level[
                                #    level_of_node[n]-2].insert(i_n,None)
                        level_of_node[p] = level_of_node[n]-1

        for l in reversed(nodes_in_level):
            for n in l[:]:
                if n is None:
                    continue
                for s in g.successors(n):
                    if s in single_dep_nodes:
                        i_n = l.index(n)
                        if level_of_node[n] >= number_of_levels-1:
                            nodes_in_level.append([])
                            number_of_levels += 1
                        #nodes_in_level[level_of_node[n]+1].insert(i_n, s)
                        if (len(nodes_in_level[level_of_node[n] + 1])
                                > i_n and
                                nodes_in_level[level_of_node[n] + 1][i_n]
                                is None):
                            nodes_in_level[level_of_node[n]+1][i_n] = s
                        else:
                            nodes_in_level[level_of_node[n]+1].insert(
                                i_n, s)
                            nodes_in_level[level_of_node[n]].insert(
                                i_n, None)
                        level_of_node[s] = level_of_node[n]+1

        for n in reversed(single_dep_nodes):
            if g.out_degree(n) != 0:
                try:
                    x = next(g.successors(n)) 
                except AttributeError: #  using networkx v1
                    x = g.successors(n)[0]
            else:  # g.in_degree(n) != 0:
                try:
                    x = next(g.predecessors(n))
                except AttributeError: #  using networkx v1
                    x = g.predecessors(n)[0]

            i_n = nodes_in_level[level_of_node[n]].index(n)
            i_x = nodes_in_level[level_of_node[x]].index(x)

            nodes_in_level[level_of_node[n]][i_n] = None
            if (len(nodes_in_level[level_of_node[n]]) > i_x and
                    nodes_in_level[level_of_node[n]][i_x] is None):
                nodes_in_level[level_of_node[n]][i_x] = n
            elif (len(nodes_in_level[level_of_node[n]]) > i_x-1 and
                    nodes_in_level[level_of_node[n]][i_x-1] is None):
                nodes_in_level[level_of_node[n]][i_x-1] = n
            elif (len(nodes_in_level[level_of_node[n]]) > i_x+1 and
                    nodes_in_level[level_of_node[n]][i_x+1] is None):
                nodes_in_level[level_of_node[n]][i_x+1] = n
            else:
                nodes_in_level[level_of_node[n]].insert(i_x, n)

        # fill index_of_node
        for l in nodes_in_level:
            for n in l:
                if n is not None:
                    index_of_node[n] = l.index(n)

        # center levels
        max_level_len = max([len(l) for l in nodes_in_level])
        width_list[layout_i] = max_level_len
        for l in nodes_in_level:
            levlen = len(l)
            l[0:] = [None]*((max_level_len-levlen)//2+1) + l + \
                [None]*((max_level_len-levlen)//2)

        # fill index_of_node
        for l in nodes_in_level:
            i_n = 0
            for n in l:
                if n is not None:
                    index_of_node[n] = i_n
                i_n += 1

        for k in range(10):
            # swap nodes
            for i in range(2):  # 6
                l_i = 0  # list index
                for l in nodes_in_level:
                    for n in l[1:]:
                        if n is None:
                            continue
                        left = l[index_of_node[n]-1]
                        suc = [x for x in graph.successors(n)
                               if level_of_node[x]-l_i < 2]
                        if left is None:
                            continue
                        left_suc = [x for x in graph.successors(left)
                                    if level_of_node[x]-l_i < 2]

                        cross_count = 0
                        cross_count_swap = 0
                        for s in suc:
                            cross_count += len(
                                [x for x in left_suc
                                 if index_of_node[x] > index_of_node[s]])
                            cross_count_swap += len(
                                [x for x in left_suc
                                 if index_of_node[x] < index_of_node[s]])

                        # swap nodes if it results in less crossings
                        if cross_count_swap < cross_count:

                            l[index_of_node[n]] = left
                            l[index_of_node[left]] = n

                            index_tmp = index_of_node[left]
                            index_of_node[left] = index_of_node[n]
                            index_of_node[n] = index_tmp

                    l_i += 1

            # swap nodes with None neighbours
            for i in range(2):  # 6
                break_flag = True
                l_i = 0  # list index
                for l in nodes_in_level:
                    for kk in range(len(l)//2):
                        break_flag = True
                        for n in l[:]:
                            if n is None:
                                continue
                            i_n = l.index(n)
                            if i_n == 0:  # first element
                                left = None
                            else:
                                left = l[i_n-1]
                            if i_n == len(l)-1:  # last element
                                right = None
                            else:
                                right = l[i_n+1]
                            suc = [x for x in graph.successors(n)
                                   if level_of_node[x]-l_i < 2]
                            if left is not None and right is not None:
                                continue

                            mean_neighbour_index = 0  # nodes_tmp.index(n)
                            count = 0.0
                            for x in multi_dep_neighbours[n]:
                                if True:  # x in g.successors(n):
                                    if abs(level_of_node[n] -
                                           level_of_node[x]) < 2:
                                        mean_neighbour_index += \
                                            nodes_in_level[
                                                level_of_node[x]].index(x)
                                        count += 1
                            if count == 0:
                                continue
                            mean_neighbour_index /= count

                            # swap nodes for being closer to
                            # mean_neighbour_index
                            if (mean_neighbour_index < i_n-.5 and
                                    left is None):
                                break_flag = False
                                l[i_n] = None
                                l[i_n-1] = n
                                index_of_node[n] = i_n - 1
                            elif (mean_neighbour_index > i_n+.5 and
                                    right is None):
                                break_flag = False
                                l[i_n] = None
                                if i_n+1 >= len(l):
                                    l.append(n)
                                else:
                                    l[i_n+1] = n
                                index_of_node[n] = i_n + 1

                        if break_flag is True:
                            break

                    l_i += 1

                if break_flag is True:
                    break

        if global_tasks_in_first_row is True:
            for n in g.nodes():
                if level_of_node[n] != 0:
                    if g.in_degree(n) == 0:
                        nodes_in_level[level_of_node[n]].remove(n)
                        #nodes_in_level[0].insert(n,index_of_node[n])
                        nodes_in_level[0].append(n)
                        level_of_node[n] = 0
            for n in nodes_in_level[0]:
                if n is not None:
                    index_of_node[n] = nodes_in_level[0].index(n)

        # build layout
        if nodes_in_level[0].count(None) == len(nodes_in_level[0]):
            offset = 1
        else:
            offset = 0
        for l_i in range(number_of_levels):
            for n in nodes_in_level[l_i]:
                if n is None:  # or isinstance(n, str):
                    continue
                x = index_of_node[n] * node_separation
                y = (-l_i + offset) * node_separation
                layout_tmp[n] = (x, y)

        height_list[layout_i] = number_of_levels
        layout_list.append(layout_tmp)

    return layout_list
