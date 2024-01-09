from collections import defaultdict
from z3 import *

def connectivity(s, vertices, edges, basepoints=None, acyclic=False):
    '''
    Compute connected components of a subgraph of a graph (V, E)
    where the subgraph is given by z3 bools.

    s: z3 Solver
    vertices: dict from V to z3 bool for whether each vertex is included in the subgraph
    edges: dict from tuple pairs (V, V) to z3 bool for whether each edge is included in the subgraph.  an edge incident to an inactive vertex will always be considered inactive regardless of its value.
    basepoints (optional): list of basepoints from which to compute distances.  each connected component will be constrained to have exactly one basepoint.  component identifiers will be assigned in iteration order over the basepoints.  each basepoint will be automatically constrained to be included.
    if basepoints is not provided then component identifiers and basepoints are assigned using iteration order on vertices.
    acyclic (optional): if True, then constrain the subgraph to be acyclic.

    return: ncc, component, distance
    ncc: number of connected components
    component: dict from V to z3 ints.  each included vertex is mapped to an identifier for its connected component.  excluded vertices are mapped to -1.
    distance: shortest distances in the subgraph from each included vertex to a basepoint in its connected component.  excluded vertices are mapped to -1.
    '''
    component = {v: FreshInt() for v in vertices}
    distance = {v: FreshInt() for v in vertices}
    ncc = FreshInt()

    # vertices
    for v, active in vertices.items():
        s.add(Implies(active, And(
            0 <= component[v], component[v] < ncc,
            0 <= distance[v], distance[v] < len(vertices),
        )))
        s.add(Implies(Not(active), And(component[v] == -1, distance[v] == -1)))

    # edges
    for (u, v), active in edges.items():
        s.add(Implies(And(vertices[u], vertices[v], active),
                      component[u] == component[v]))

    # number of connected components
    s.add(Or(
        ncc == 0,
        Or([And(active, ncc == component[v] + 1)
            for v, active in vertices.items()]),
    ))

    # basepoint and component identifiers
    if basepoints is None:
        v_order = list(vertices)
        # canonical component identifier choices
        for (i, v) in enumerate(v_order):
            lesser = [v_order[j] for j in range(i)]
            s.add(Or(
                component[v] <= 0,
                Or([component[v] == component[v2] for v2 in lesser]),
                Or([component[v] == component[v2] + 1 for v2 in lesser]),
            ))
            for v2 in lesser:
                # canonical unique basepoint
                s.add(Implies(distance[v] == 0, component[v2] != component[v]))
    else:
        basepoints = list(basepoints)
        s.add(ncc == len(basepoints))
        for (i, v) in enumerate(basepoints):
            s.add(component[v] == i)
            s.add(distance[v] == 0)
        for v in vertices:
            if v not in basepoints:
                s.add(distance[v] != 0)


    # record active neighbors via active edges
    neighbors = defaultdict(list)
    for (u, v), active in edges.items():
        neighbors[u].append((v, And(vertices[v], active)))
        neighbors[v].append((u, And(vertices[u], active)))

    # non-root distance is min of dist+1 of neighbors
    for u in vertices:
        s.add(Implies(
            distance[u] > 0,
            And(
                And([Implies(active, distance[u] <= distance[v]+1)
                     for v, active in neighbors[u]]),
                Or([And(active, distance[u] == distance[v]+1)
                    for v, active in neighbors[u]]),
            )))

    # acyclicity
    if acyclic:
        for u in vertices:
            # exactly 1 neighbor of non-root u has distance at most distance[u]
            s.add(Implies(
                distance[u] > 0,
                Sum([If(And(active, distance[v] <= distance[u]), 1, 0)
                     for v, active in neighbors[u]] == 1)))

    return ncc, component, distance
