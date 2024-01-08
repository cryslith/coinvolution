# Generalized maps

# Main ideas are from
# https://doc.cgal.org/latest/Generalized_map/index.html

from abc import  ABC, abstractmethod
from collections.abc import MutableMapping
import itertools

import jsonschema


# An alpha-list a is a tuple (alphas, d) where 
# alphas is a repeatable iterable of alpha indices.
# If d is not None, implicitly include all higher indices starting at d.


class Alphas:
    '''set of alpha indices represented as bitstring'''
    def __init__(self, bits):
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
        '''return iterator over all darts.  darts must be immutable, hashable, and compararable (in the same order as returned by this method)'''
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

    def shared_dart_per_incident_orbit(self, dart, a, b):
        '''
        one dart per a-orbit incident to self's b-orbit.
        darts are guaranteed to be in both orbits.

        a, b: alpha-list
        '''
        return self.unique_by_orbit(self.orbit(dart, b), a)

    def rep(self, dart, a):
        '''Obtain dart's a-rep: the minimum element of dart's a-orbit.'''
        return min(self.orbit(dart, a))

    def vertex(self, dart):
        return self.rep(dart, Alphas.VERTEX)
    def edge(self, dart):
        return self.rep(dart, Alphas.EDGE)
    def face(self, dart):
        return self.rep(dart, Alphas.FACE)

    def rep_per_orbit(self, l, a):
        '''
        returns a-rep of each dart in iterator l.
        only one item is returned for each a-orbit.

        a: Alphas
        '''
        seen = set()
        for dart in l:
            if dart in seen:
                continue
            b = dart
            for n in self.orbit(dart, a):
                if n < b:
                    b = n
                seen.add(n)
            yield b

    def rep_per_incident_orbit(self, dart, a, b):
        '''
        a-rep of each a-orbit incident to dart's b-orbit.

        a, b: alpha-list
        '''
        return self.rep_per_orbit(self.orbit(dart, b), a)
