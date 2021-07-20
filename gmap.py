# Generalized maps

# Follows the description at
# https://doc.cgal.org/latest/Generalized_map/index.html
# and adapts code from CGAL too

from collections.abc import MutableMapping
import itertools


# An alpha-list a is a tuple (alphas, d) where 
# alphas is a repeatable iterable of alpha indices.
# If d is not None, implicitly include all higher indices starting at d.


def unique_by_orbit(l, a):
    '''
    filters iterator l down to one dart per a-orbit

    a: alpha-list
    '''
    seen = set()
    for dart in l:
        if dart in seen:
            continue
        yield dart
        for n in dart.orbit(a):
            seen.add(n)


def cell_alphas(i, dim=None):    
    if dim is None:
        return (list(range(i)), i + 1)
    else:
        return ([j for j in range(dim + 1) if j != i], None)


class Dart:
    def __init__(self, dimension, number):
        self.alpha = [self] * (dimension + 1)
        self.number = number

    def increase_dimension(self, dim):
        if dim < len(self.alpha) - 1:
            raise ValueError('cannot lower dimension')
        self.alpha.extend([self] * (dim - (len(self.alpha) - 1)))

    def al(self, *ii):
        d = self
        for i in ii:
            d = d.alpha[i]
        return d

    def _alphas(self, a):
        '''
        list of actual alpha values

        a: alpha-list
        '''
        (alphas, d) = a
        if d is None:
            return alphas
        else:
            return list(itertools.chain(alphas, range(d, len(self.alpha))))

    def orbit_paths(self, a):
        '''
        iterator over the orbit of self under a.
        returns iterator of ([alpha_indices], dart)
        where alpha_indices is the path of alpha indices
        from self to the new dart.
        always includes ((), self).

        a: alpha-list
        '''
        seen = set()
        frontier = [((), self)]
        while frontier:
            (path, dart) = frontier.pop(0)
            if dart in seen:
                continue
            seen.add(dart)
            yield (path, dart)
            for i in self._alphas(a):
                neighbor = dart.alpha[i]
                frontier.append((path + (i,), neighbor))

    def orbit(self, alphas):
        return (dart for _, dart in self.orbit_paths(alphas))

    def cell(self, i, dim=None):
        return self.orbit(cell_alphas(i, dim))

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
        '''
        sew self's i-cell along other's i-cell.
        returns list of pairs of darts sewn
        '''
        alphas = ([j for j in range(len(self.alpha)) if abs(j - i) > 1], None)
        m1 = dict(self.orbit_paths(alphas))
        m2 = dict(other.orbit_paths(alphas))
        if m1.keys() != m2.keys():
            raise ValueError('unsewable')
        output = []
        for (k, d1) in m1.items():
            d2 = m2[k]
            d1._link(i, d2)
            output.append((d1, d2))
        return output

    def unsew(self, i):
        '''returns list of pairs of darts unsewn'''
        alphas = ([j for j in range(len(self.alpha)) if abs(j - i) > 1], None)
        output = []
        for d1 in self.orbit(alphas):
            d2 = d1._unlink(i)
            output.append((d1, d2))
        return output

    def one_dart_per_incident_orbit(self, a, b):
        '''
        one dart per a-orbit incident to self's b-orbit.
        darts are guaranteed to be in both orbits.

        a, b: alpha-list
        '''
        return unique_by_orbit(self.orbit(b), a)


    def one_dart_per_incident_cell(self, i, j, dim=None):
        '''
        one dart per i-cell (in dim) incident to self's j-cell (in dim).
        darts are guaranteed to be in both cells.
        '''
        return self.one_dart_per_incident_orbit(cell_alphas(i, dim), cell_alphas(j, dim))

    def __str__(self):
        return '{:3}'.format(self.number)

    def __repr__(self):
        return 'Dart({}, {})'.format(self.number, [x.number for x in self.alpha])


class GMap:
    def __init__(self, dimension, darts=()):
        '''
        dimension: dimension of each dart
        darts: iterable of darts
        '''
        self.dimension = dimension
        self.darts = list(darts)

    def increase_dimension(self, dim):
        if dim < self.dimension:
            raise ValueError('cannot lower dimension')
        self.dimension = dim
        for d in self.darts:
            d.increase_dimension(dim)

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
        self.darts.append(d)
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

    def one_dart_per_orbit(self, a):
        return unique_by_orbit(self.darts, a)

    def one_dart_per_cell(self, i, dim=None):
        '''one dart per i-cell (in dim)'''
        return self.one_dart_per_orbit(cell_alphas(i, dim))

    def serialize(self):
        darts = [[n.number for n in d.alpha] for d in self.darts]
        return {'dimension': self.dimension, 'darts': darts}


class Grid(GMap):
    '''
    n * m grid.  n rows, m columns.
    rows increase from north to south,
    columns increase from west to east
    '''
    def __init__(self, n, m):
        super().__init__(2)
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
        # Each square is the dart on the square's north edge, northwest vertex
        self.squares = rows

    def vertex_grid(self):
        vrows = []
        for row in self.squares:
            vrow = []
            for d in row:
                vrow.append(d)
            vrow.append(d.al(0))
            vrows.append(vrow)
        lastvrow = []
        for d in row:
            lastvrow.append(d.al(1, 0, 1))
        lastvrow.append(d.al(1, 0, 1, 0))
        vrows.append(lastvrow)
        return vrows

    def vertex_positions(self):
        pos = CellDict(0)
        for i, row in enumerate(self.vertex_grid()):
            for j, d in enumerate(row):
                pos[d] = (j, i)
        return pos


class OrbitDict(MutableMapping):
    '''Dictionary over generalized cells (orbits of alphas)'''

    def __init__(self, a):
        self.darts = {}
        self.a = a

    @classmethod
    def over_cells(cls, i, dim=None):
        '''Dictionary over i-cells in dim'''
        return cls(cell_alphas(i, dim))

    def __getitem__(self, dart):
        return self.darts[dart]

    def __setitem__(self, dart, value):
        for d in dart.orbit(self.a):
            self.darts[d] = value

    def __delitem__(self, dart):
        for d in dart.orbit(self.a):
            del self.darts[d]

    def __iter__(self):
        return unique_by_orbit(self.darts, self.a)

    def __len__(self):
        raise TypeError

    def resolve_sew(self, sew_list, merge=None):
        '''
        fix up the mapping to account for a sewing.
        sew_list is a list of darts sewn.
        merge function is used to merge pairs of values if both are present
        (default is take left)
        '''
        if merge is None:
            merge = lambda x, _: x
        for (d1, d2) in sew_list:
            if d1 in self.darts:
                if d2 in self.darts:
                    v = merge(self.darts[d1], self.darts[d2])
                    self.darts[d1] = v
                    self.darts[d2] = v
                else:
                    self.darts[d2] = self.darts[d1]
            else:
                if d2 in self.darts:
                    self.darts[d1] = self.darts[d2]

    def serialize(self):
        darts = {d.number: v for d, v in self.darts.items()}
        return {'alpha-list': self.a, 'darts': darts}


def CellDict(i, dim=None):
    return OrbitDict.over_cells(i, dim)
