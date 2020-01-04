# Generalized maps

# Follows the description at
# https://doc.cgal.org/latest/Generalized_map/index.html
# and adapts code from CGAL too

def group_by_cell(l, i, dim=None):
    '''groups iterator l into i-cells (in dim)'''
    seen = set()
    for dart in l:
        if dart in seen:
            continue
        cell = []
        for n in dart.cell(i, dim):
            cell.append(n)
            seen.add(n)
        yield cell

def unique_by_cell(l, i, dim=None):
    '''
    filters iterator l down to one dart per i-cell, considered in dimension dim
    '''
    seen = set()
    for dart in l:
        if dart in seen:
            continue
        yield dart
        for n in dart.cell(i, dim):
            seen.add(n)

class Dart:
    def __init__(self, dimension, number):
        self.alpha = [self] * (dimension + 1)
        self.number = number # for debugging

    def al(self, *ii):
        d = self
        for i in ii:
            d = d.alpha[i]
        return d

    def orbit_paths(self, alphas):
        '''
        iterator over the orbit of a dart under alphas.
        returns iterator of ([alpha_indices], dart)
        where alpha_indices is the path of alpha indices
        from self to the new dart.
        always includes ((), self).

        alphas: a repeatable iterable of alpha indices
        '''
        seen = set()
        frontier = [((), self)]
        while frontier:
            (path, dart) = frontier.pop(0)
            if dart in seen:
                continue
            seen.add(dart)
            yield (path, dart)
            for i in alphas:
                neighbor = dart.alpha[i]
                frontier.append((path + (i,), neighbor))

    def orbit(self, alphas):
        return (dart for _, dart in self.orbit_paths(alphas))

    def cell_paths(self, i, dim=None):
        '''
        iterator over all darts in the same i-cell as this one.
        same return type as orbit_indices.
        the cell is considered in dimension dim
        (default to the overall dimension of the map)
        '''
        if dim is None:
            dim = len(self.alpha) - 1
        alphas = list(j for j in range(dim + 1) if j != i)
        return self.orbit_paths(alphas)

    def cell(self, i, dim=None):
        return (dart for _, dart in self.cell_paths(i, dim))

    def _link(self, i, other):
        if not self.is_free(i):
            raise ValueError('not free')
        self.alpha[i] = other
        other.alpha[i] = self

    def _unlink(self, i):
        if self.is_free(i):
            raise ValueError('already free')
        other = self.alpha[i]
        other.alpha[i] = other
        self.alpha[i] = self
        return other

    def is_free(self, i):
        return self.alpha[i] == self

    def sew(self, i, other):
        alphas = list(j for j in range(len(self.alpha))
                      if abs(j - i) > 1)
        m1 = dict(self.orbit_paths(alphas))
        m2 = dict(other.orbit_paths(alphas))
        if m1.keys() != m2.keys():
            raise ValueError('unsewable')
        for (k, d1) in m1.items():
            d2 = m2[k]
            d1._link(i, d2)

    def unsew(self, i):
        other = self.alpha[i]
        alphas = list(j for j in range(len(self.alpha))
                      if abs(j - i) > 1)
        for d in self.orbit(alphas):
            d._unlink(i)
        return other

    def one_dart_per_incident_cell(self, i, j, dim=None):
        '''
        one dart per i-cell (in dim) incident to self's j-cell (in dim)
        '''
        return unique_by_cell(self.cell(j, dim), i, dim)

    def __str__(self):
        return '{:3}'.format(self.number)

    def __repr__(self):
        return 'Dart({})'.format(self.number)


class GMap:
    def __init__(self, dimension, darts=()):
        '''
        dimension: dimension of each dart
        darts: iterable of darts
        '''
        self.dimension = dimension
        self.darts = set(darts)

    def check_validity(self):
        for dart in self.darts:
            if len(dart.alpha) - 1 != self.dimension:
                raise ValueError('dart {} has dimension {}, expected {}'
                                 .format(dart, len(dart.alpha) - 1, self.dimension))

        for i in range(self.dimension + 1):
            for dart in self.darts:
                if dart.alpha[i].alpha[i] != dart:
                    raise ValueError('alpha_{} is not an involution'.format(i))

        for i in range(self.dimension - 1):
            for j in range(i + 2, self.dimension + 1):
                for dart in self.darts:
                    if dart.alpha[i].alpha[j] != dart.alpha[j].alpha[i]:
                        raise ValueError('alpha_{} alpha_{} is not an involution'
                                         .format(i, j))

    def create_dart(self):
        d = Dart(self.dimension, len(self.darts))
        self.darts.add(d)
        return d

    def make_edge(self):
        d = self.create_dart()
        d._link(0, self.create_dart())
        return d

    def make_polygon(self, n):
        start = self.make_edge()
        prev = start.alpha[0]
        for _ in range(n - 1):
            c = self.make_edge()
            c._link(1, prev)
            prev = c.alpha[0]
        start._link(1, prev)
        return start

    def make_tetrahedron(self):
        d0 = self.make_polygon(3)
        d1 = self.make_polygon(3)
        d2 = self.make_polygon(3)
        d3 = self.make_polygon(3)

        d0.sew(2, d1)
        d0.al(0, 1).sew(2, d2)
        d0.al(1, 0).sew(2, d3)

        d1.al(0, 1).sew(2, d2.al(1))
        d2.al(0, 1).sew(2, d3.al(1))
        d3.al(0, 1).sew(2, d1.al(1))
        return d0

    def make_cube(self):
        bottom = self.make_polygon(4)
        top = self.make_polygon(4)
        sides = [self.make_polygon(4) for _ in range(4)]
        b = bottom
        t = top

        for s in sides:
            bottom.sew(2, s)
            bottom = bottom.al(0, 1)
            top.sew(2, s.al(1, 0, 1))
            top = top.al(0, 1)

        for (s0, s1) in zip(sides, sides[1:] + [sides[0]]):
            s0.al(0, 1).sew(2, s1.al(1))

        return bottom

    def one_dart_per_cell(self, i, dim=None):
        '''one dart per i-cell (in dim)'''
        return unique_by_cell(self.darts, i, dim)

    def all_cells(self, i, dim=None):
        '''darts grouped by i-cell (in dim)'''
        return group_by_cell(self.darts, i, dim)
