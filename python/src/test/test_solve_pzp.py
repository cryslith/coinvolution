from unittest import TestCase

from ..gmap import Alphas
from ..solvers import pzp_solvers
from ..pzp import decode

PZP_SOLVERS = pzp_solvers()

class TestSolvePZP(TestCase):
    def test_simpleloop(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?simpleloop/5/5/o2c1v')
        solver = PZP_SOLVERS[variety](g, layers, extra)
        [(sol_layers, sol_extra)] = solver.solutions()
        self.assertIsNone(extra)
        self.assertIsNone(sol_extra)
        [edges] = sol_layers
        self.assertEqual(edges.name, 'edges')
        self.assertEqual(edges.alphas, Alphas.EDGE)
        self.assertEqual(
            {k for (k, v) in edges.data.items() if v},
            {g.e_top(y, x) for (y, x) in [
                (1, 2),
                (1, 4),
                (2, 0),
                (2, 4),
                (3, 0),
                (3, 3),
            ]} |
            {g.e_right(y, x) for (y, x) in [
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 1),
                (2, 3),
                (3, 0),
                (3, 1),
                (3, 2),
            ]},
        )

    def test_numberlink(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?numlin/6/5/g4i4123r123.j4')
        solver = PZP_SOLVERS[variety](g, layers, extra)
        [(sol_layers, sol_extra)] = solver.solutions()
        self.assertIsNone(extra)
        self.assertIsNone(sol_extra)
        [edges] = sol_layers
        self.assertEqual(edges.name, 'edges')
        self.assertEqual(edges.alphas, Alphas.EDGE)
        self.assertEqual(
            {k for (k, v) in edges.data.items() if v},
            {g.e_top(y, x) for (y, x) in [
                (2, 0), (2, 1), (2, 5),
                (3, 0), (3, 4), (3, 5),
            ]} |
            {g.e_right(y, x) for (y, x) in [
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 1), (2, 2), (2, 3),
                (3, 0), (3, 1), (3, 2),
                (4, 0), (4, 1), (4, 2), (4, 3), (4, 4),
            ]},
        )

    def test_forced_vars(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?simpleloop/6/4/gg022')
        solver = PZP_SOLVERS[variety](g, layers, extra)
        for _ in solver.solutions():
            pass
        ([edges], _) = solver.forced_variables
        self.assertEqual(
            {k for (k, v) in edges.data.items() if v},
            {g.e_top(y, x) for (y, x) in [(1, 1), (1, 4), (2, 0), (2, 5), (3, 1), (3, 4)]} |
            {g.e_right(y, x) for (y, x) in [(0, 1), (0, 3), (1, 0), (1, 4), (2, 0), (2, 4), (3, 1), (3, 3)]},
        )
