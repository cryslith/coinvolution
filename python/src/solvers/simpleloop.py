from ..gmap import *
from ..puzzle import Layer
from . import PSolver
from .util import connectivity

from z3 import *

class S(PSolver):
    solver = None

    def __init__(self, g, layers, extra=None):
        layers = {l.name: l for l in layers}
        shaded = layers['shaded'].data

        s = Solver()
        edges = {}
        for e in g.edges():
            edges[e] = FreshBool()
            fs = list(g.rep_per_incident_orbit(e, Alphas.FACE, Alphas.EDGE))
            if len(fs) < 2 or any(shaded.get(f) for f in fs):
                s.add(edges[e] == False)
        for f in g.faces():
            degree = Sum([If(edges[e], 1, 0)
                           for e in g.rep_per_incident_orbit(f, Alphas.EDGE, Alphas.FACE)])
            if shaded.get(f):
                s.add(degree == 0)
            else:
                s.add(degree == 2)

        ncc, _, _ = connectivity(
            s,
            {f: not shaded.get(f) for f in g.faces()},
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
            output_edges[e] = bool(m[self.edges[e]])

        return [Layer('edges', Alphas.EDGE, output_edges)], []
