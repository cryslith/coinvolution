from unittest import TestCase

from ..solve_pzp import solutions
from ...gmap import Alphas
from ...puzzle import Display

class TestSolvePZP(TestCase):
    def test_solve_pzp(self):
        (variety, g, layers, extra, solns) = list(solutions('https://puzz.link/p?simpleloop/5/5/o2c1v'))
        self.assertIsNone(extra)
        [(sol_layers, sol_extra)] = solns
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
            ]}
        )
