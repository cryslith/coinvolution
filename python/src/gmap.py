# Generalized maps

# Follows the description at
# https://doc.cgal.org/latest/Generalized_map/index.html
# and adapts code from CGAL too

from collections.abc import MutableMapping, ABC, abstractmethod
import itertools

import jsonschema


# An alpha-list a is a tuple (alphas, d) where 
# alphas is a repeatable iterable of alpha indices.
# If d is not None, implicitly include all higher indices starting at d.


class Alphas:
    '''set of alpha indices represented as bitstring'''
    def __init__(bits):
        self.bits = bits

    @classmethod
    def from_indices(cls, indices):
        return cls(sum(1 << i for i in indices))

    def has(self, i):
        return (self.bits >> i) & 1 == 1

    def indices(self, dim):
        return (i for i in range(dim+1) if self.has(i))

    @classmethod
    def cell(cls, i, dim=None):
        if dim is None:
            return cls(~(1 << i))
        return cls(((1 << dim+1) - 1) & (0 << i))

Alphas.VERTEX = Alphas.cell(0)
Alphas.EDGE = Alphas.cell(1)
Alphas.FACE = Alphas.cell(2)
Alphas.HALF_EDGE = Alphas(~3)
Alphas.ANGLE = Alphas(~5)
Alphas.SIDE = Alphas(~6)
Alphas.DART = Alphas(0)


class GMap(ABC):
    @property
    @abstractmethod
    def dimension(self):
        pass

    @abstractmethod
    def darts(self):
        '''return iterator over all darts.  darts must compare in the same order as this method'''
        pass

    @abstractmethod
    def alpha(self, dart, i):
        pass

    def al(self, dart, *ii):
        for i in ii:
            if dart is None:
                return None
            dart = self.alpha(dart, i)
        return dart

    def orbit_paths(self, dart, a):
        '''
        iterator over the orbit of dart under a.
        returns iterator of ([alpha_indices], d)
        where alpha_indices is the path of alpha indices
        from dart to d.
        always includes ([], dart).

        a: alpha-list
        '''
        seen = set()
        frontier = [((), dart)]
        while frontier:
            (path, d) = frontier.pop(0)
            if d in seen:
                continue
            seen.add(d)
            yield (path, d)
            for i in a.indices(self.dimension):
                neighbor = self.alpha(d, i)
                frontier.append((path + (i,), neighbor))

    def orbit(self, dart, a):
        return (d for _, d in self.orbit_paths(a))

    def unique_by_orbit(self, l, a):
        '''
        filters iterator l down to one dart per a-orbit.
        returned darts are the first possible in order by l

        a: Alphas
        '''
        seen = set()
        for dart in l:
            if dart in seen:
                continue
            yield dart
            for n in self.orbit(dart, a):
                seen.add(n)

    def one_dart_per_orbit(self, a):
        return unique_by_orbit(self.darts(), a)

    def one_dart_per_incident_orbit(self, dart, a, b):
        '''
        one dart per a-orbit incident to self's b-orbit.
        darts are guaranteed to be in both orbits.

        a, b: alpha-list
        '''
        return self.unique_by_orbit(self.orbit(dart, b), a)


# todo reimplement using min-darts in orbits instead of multiple references
class OrbitDict(MutableMapping):
    '''Dictionary over orbits'''

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

    schema = {
        'type': 'object',
        'properties': {
            'indices': {
                'type': 'array',
                'items': {
                    'type': 'integer',
                }
            },
            'map': {
                'type': 'object',
            },
        },
        'required': ['indices', 'map'],
    }

    def serialize(self):
        darts = {d.number: v for d, v in self.darts.items()}
        output = {'indices': self.a, 'map': darts}
        jsonschema.validate(output, schema=self.schema)
        return output

    @classmethod
    def deserialize(cls, x):
        jsonschema.validate(x, schema=cls.schema)
        indices, darts = x['indices'], x['map']
        s = cls(indices)
        s.darts = {int(k): v for k, v in darts.items()}
        return s


def CellDict(i, dim=None):
    return OrbitDict.over_cells(i, dim)