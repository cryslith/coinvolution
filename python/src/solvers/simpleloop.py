from ..gmap import *
from ..puzzle import Layer, Display
from . import Z3Solver
from .util import connectivity

from z3 import *

class S(Z3Solver):
    solver = None

    def __init__(self, g, layers, extra=None):
        layers = {l.name: l for l in layers}
        shaded = {k for (k, v) in layers['shaded'].data.items() if v}

        s = Solver()
        edges = {}
        for e in g.edges():
            edges[e] = FreshBool()
            fs = list(g.rep_per_incident_orbit(e, Alphas.FACE, Alphas.EDGE))
            if len(fs) < 2 or any(f in shaded for f in fs):
                s.add(edges[e] == False)
        for f in g.faces():
            degree = Sum([If(edges[e], 1, 0)
                           for e in g.rep_per_incident_orbit(f, Alphas.EDGE, Alphas.FACE)])
            if f in shaded:
                s.add(degree == 0)
            else:
                s.add(degree == 2)

        ncc, _, _ = connectivity(
            s,
            {f: f not in shaded for f in g.faces()},
            {fs: edges[e] for e in g.edges() if len(fs := tuple(g.rep_per_incident_orbit(e, Alphas.FACE, Alphas.EDGE))) == 2},
        )
        s.add(ncc == 1)

        self.g = g
        self.solver = s
        self.edges = edges

    def vars(self):
        return self.edges.values()

    def model_to_layers(self, m):
        output_edges = {}
        for e in self.g.edges():
            output_edges[e] = 1 if m[self.edges[e]] else 0

        return [Layer('edges', Alphas.EDGE, output_edges, Display.line)], None
