#!/usr/bin/env python3

from z3 import *
from gmap import *

def other_face(face, edge):
    for f in edge.one_dart_per_incident_cell(2, 1):
        if face not in f.cell(2):
            return face

def solve(g, layers, extra=None):
    s = Solver()
    edges = CellDict(1, 2)
    for e in g.one_dart_per_cell(1):
        edges[e] = FreshBool()
        if len(list(e.one_dart_per_incident_cell(2, 1))) != 2:
            s.add(edges[e] == False)
    for f in g.one_dart_per_cell(2):
        s.add(Sum([If(edges[e], 1, 0)
                   for e in f.one_dart_per_incident_cell(1, 2)]) == 2)

    def connectivity():
        faces = list(g.one_dart_per_cell(2))
        root = faces[0]
        dists = CellDict(2, 2)
        for f in faces:
            dists[f] = FreshInt()
        infinity = len(faces)

        for f in faces:
            d = dists[f]
            if f == root:
                s.add(d == 0)
            else:
                s.add(And([d > 0, d < infinity]))
                neighbor_dists = [If(edges[e], dists[other_face(f, e)], infinity) for e in f.one_dart_per_incident_cell(1, 2) if other_face(f, e) is not None]
                s.add(Or([d == n for n in neighbor_dists]))
                s.add(And([d <= n + 1 for n in neighbor_dists]))
    # connectivity()

    output_edges = CellDict(1, 2)
    s.check()
    m = s.model()
    for e in g.one_dart_per_cell(1):
        output_edges[e] = bool(m[edges[e]])

    return {
        'layers': [
            {
                'name': 'edges',
                'type': 'enum',
                'data': output_edges,
            }
        ]
    }
