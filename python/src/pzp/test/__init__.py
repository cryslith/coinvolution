from unittest import TestCase

from .. import decode

class TestPZP(TestCase):
    def test_decode_slither(self):
        s = 'https://puzz.link/p?slither/7/4/01239.g56cgdjah'
        (variety, g, layers) = decode(s)
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
