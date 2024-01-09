from unittest import TestCase

from ..solve_pzp import solutions

class TestSolvePZP(TestCase):
    def test_solve_pzp(self):
        self.assertEqual(1, sum(1 for _ in solutions('https://puzz.link/p?simpleloop/5/5/o2c1v')))
