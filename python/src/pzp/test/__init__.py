from unittest import TestCase

from .. import decode

class TestPZPDecode(TestCase):
    def test_slither(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?slither/7/4/01239.g56cgdjah')
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
        self.assertIsNone(extra)

    def test_simpleloop(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?simpleloop/5/5/o2c1v')
        [shaded] = layers
        self.assertEqual(shaded.name, 'shaded')
        self.assertEqual(
            {k for (k, v) in shaded.data.items() if v},
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
        self.assertIsNone(extra)

    def test_yajilin(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?yajilin/b/40/6/4132a23a10zh0.a0.0.a00zi01a02a03zh511zm912zzza'.replace('/b', '')) # todo support this b thingy
        # todo

    def test_numlin(self):
        (variety, g, layers, extra) = decode('https://puzz.link/p?numlin/6/3/-beg+3e7j+1f32-22g-11h.i')
        self.assertEqual(variety, 'numlin')
        [clues] = layers
        self.assertEqual(clues.data,
            {g[y, x]: v for (y, x, v) in [
                (0, 0, 190),
                (0, 2, 999),
                (1, 1, 499),
                (1, 2, 2),
                (1, 3, 34),
                (1, 5, 17),
                (2, 2, '?'),
            ]},
        )
