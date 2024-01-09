from . import GMap

import itertools

class Grid(GMap):
    dimension = 2

    '''
    n * m grid; n rows, m columns.
    rows increase from north to south,
    columns increase from west to east

    darts are of the form (y, x, i) where y and x give the coordinates of the square.  i=0 for the top-left dart in the square, i=1 for the top-right dart, i=2 for the right-up dart, etc
    '''
    def __init__(self, n, m):
        self.height = n
        self.width = m

    def has_square(self, y, x):
        return 0 <= y < self.height and 0 <= x < self.width

    def darts(self):
        return ((y, x, i) for y in range(self.height) for x in range(self.width) for i in range(8))

    def alpha(self, dart, j):
        (y, x, i) = dart
        if j == 0:
            return (y, x, i ^ 1)
        if j == 1:
            return (y, x, (((i + 1) ^ 1) + 7) % 8)
        if j == 2:
            yo = (-1, -1, 0, 0, 1, 1, 0, 0)[i]
            xo = (0, 0, 1, 1, 0, 0, -1, -1)[i]
            y += yo
            x += xo
            if self.has_square(y, x):
                return (y, x, ((i ^ 1) + 4) % 8)
            return None
        raise ValueError

    def __getitem__(self, k):
        (y, x) = k
        if not self.has_square(y, x):
            raise KeyError(k)
        return (y, x, 0)

    def f_loc(self, dart):
        'location of face'
        (y, x, _) = dart
        return (y, x)

    def v_loc(self, dart):
        'location of vertex'
        (y, x, i) = dart
        yo = ((i + 1) % 8) // 4
        xo = ((i + 2) % 8) // 4
        return (y + yo, x + xo)

    def e_loc2(self, dart):
        'location of edge in doubled coordinates'
        (y, x, i) = dart
        yo = (0, 0, 1, 1, 2, 2, 1, 1)[i]
        xo = (1, 1, 2, 2, 1, 1, 0, 0)[i]
        return (2*y + yo, 2*x + xo)

    def e_left(self, y, x):
        return self.edge((y, x, 6))
    def e_right(self, y, x):
        return self.edge((y, x, 2))
    def e_top(self, y, x):
        return self.edge((y, x, 0))
    def e_bottom(self, y, x):
        return self.edge((y, x, 4))
    def v_tl(self, y, x):
        return self.vertex((y, x, 0))
    def v_tr(self, y, x):
        return self.vertex((y, x, 2))
    def v_bl(self, y, x):
        return self.vertex((y, x, 6))
    def v_br(self, y, x):
        return self.vertex((y, x, 4))

    def v_at_loc(self, y, x):
        if not (0 <= y <= self.height and 0 <= x <= self.width):
            raise ValueError
        if y == self.height:
            if x == self.width:
                return self.v_br(y-1, x-1)
            return self.v_bl(y-1, x)
        if x == self.width:
            return self.v_tr(y, x-1)
        return self.v_tl(y, x)

    def to_dartlist(self):
        raise NotImplementedError
        rows = []
        for _ in range(n):
            row = []
            for _ in range(m):
                row.append(self.make_polygon(4))
            for s0, s1 in zip(row, row[1:]):
                s0.al(0, 1).sew(2, s1.al(1))
            rows.append(row)
        for r0, r1 in zip(rows, rows[1:]):
            for s0, s1 in zip(r0, r1):
                s0.al(1, 0, 1).sew(2, s1)
