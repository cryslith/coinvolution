from . import GMap

class Dart:
    def __init__(self, dimension, number):
        self.alpha = [self] * (dimension + 1)
        self.number = number

    def increase_dimension(self, dim):
        if dim < len(self.alpha) - 1:
            raise ValueError('cannot lower dimension')
        self.alpha.extend([self] * (dim - (len(self.alpha) - 1)))

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

    def __str__(self):
        return '{:3}'.format(self.number)

    def __repr__(self):
        return 'Dart({}, {})'.format(self.number, [x.number for x in self.alpha])


class Dartlist(GMap):
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

    schema = {
        'type': 'object',
        'properties': {
            'dimension': {
                'type': 'integer',
            },
            'alpha': {
                'type': 'object',
                'additionalProperties': {
                    'type': 'array',
                    'items': {
                        'type': 'integer',
                    },
                },
            },
        },
        'required': ['dimension', 'alpha'],
    }

    def serialize(self):
        darts = {d.number: [n.number for n in d.alpha] for d in self.darts}
        output = {'dimension': self.dimension, 'alpha': darts}
        jsonschema.validate(output, schema=self.schema)
        return output

    @classmethod
    def deserialize(cls, x):
        jsonschema.validate(x, schema=cls.schema)
        dimension, alpha = x['dimension'], x['alpha']
        alpha = {int(k): v for (k, v) in alpha.items()}
        numbered_darts = {k: Dart(dimension, k) for k in alpha}
        for n, v in alpha.items():
            numbered_darts[n].alpha = [numbered_darts[i] for i in v]
        o = cls(dimension, numbered_darts.values())
        o.check_validity()
        return o
