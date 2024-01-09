from unittest import TestCase

from .. import decode

class TestPZP(TestCase):
    def test_decode_slither(self):
        (variety, g, layers) = decode('https://puzz.link/p?slither/7/4/01239.g56cgdjah')
        self.assertEqual(variety, 'slither')
        self.assertEqual(g.width, 7)
        self.assertEqual(g.height, 4)
        [clues] = layers
        self.assertEqual(clues.name, 'clues')
        self.assertEqual(
            clues.data,
            {g[y, x]: v for (y, x, v) in [
                (0, 0, 0),
                (0, 1, 1),
                (0, 2, 2),
                (0, 3, 3),
                (0, 4, 4),
                (1, 1, 0),
                (1, 3, 1),
                (1, 5, 2),
                (2, 2, 3),
                (3, 2, 0),
            ]},
        )

    def test_decode_simpleloop(self):
        (variety, g, layers) = decode('https://puzz.link/p?simpleloop/5/5/o2c1v')
        [shaded] = layers
        self.assertEqual(shaded.name, 'shaded')
        self.assertEqual(
            shaded.data,
            {g[y, x] for (y, x) in [
                (0, 0),
                (0, 1),
                (1, 3),
                (2, 1),
                (2, 2),
                (3, 4),
                (4, 0),
                (4, 1),
                (4, 2),
                (4, 3),
                (4, 4),
            ]},
        )
