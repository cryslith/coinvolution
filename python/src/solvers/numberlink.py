from ..gmap import *
from ..puzzle import Layer, Display
from . import Z3Solver
from .util import connectivity

from itertools import combinations

from z3 import *

class S(Z3Solver):
    solver = None

    def __init__(self, g, layers, extra=None):
        layers = {l.name: l for l in layers}
        clues = layers['clues'].data

        s = Solver()
        edges = {}
        for e in g.edges():
            edges[e] = FreshBool()
            fs = list(g.rep_per_incident_orbit(e, Alphas.FACE, Alphas.EDGE))
            if len(fs) < 2:
                s.add(Not(edges[e]))
        for f in g.faces():
            degree = Sum([If(edges[e], 1, 0)
                           for e in g.rep_per_incident_orbit(f, Alphas.EDGE, Alphas.FACE)])
            if f in clues:
                s.add(degree == 1)
            else:
                s.add(Or(degree == 0, degree == 2))

        _, component, _ = connectivity(
            s,
            {f: True for f in g.faces()},
            {fs: edges[e] for e in g.edges() if len(fs := tuple(g.rep_per_incident_orbit(e, Alphas.FACE, Alphas.EDGE))) == 2},
            acyclic=True,
        )
        for (f1, v1), (f2, v2) in combinations(((f, v) for f, v in clues.items() if v != '?'), 2):
            if v1 == v2:
                continue
            s.add(Not(component[f1] == component[f2]))

        self.g = g
        self.solver = s
        self.edges = edges

    def vars(self):
        return self.edges.values()

    def model_to_layers(self, m):
        output_edges = {e: 1 if m[self.edges[e]] else 0 for e in self.g.edges() if self.edges[e] in m}
        return [Layer('edges', Alphas.EDGE, output_edges, Display.line)], None
