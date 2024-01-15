from abc import ABC, abstractmethod
import itertools

from z3 import Or, sat, IntNumRef, BoolRef

def pzp_solvers():
    from . import simpleloop
    return {
        'simpleloop': simpleloop.S,
    }

class PSolver(ABC):
    @abstractmethod
    def __init__(self, g, layers, extra):
        pass

    def solutions(self):
        '''
        iterator over all solutions.

        undefined behavior if multiple solution iterators are created.
        '''
        s = self.solver
        while s.check() == sat:
            m = s.model()
            solution = self.model_to_layers(m)
            yield solution
            s.add(Or([v != m[v] for v in self.vars()]))

class Z3Solver(PSolver):
    @property
    @abstractmethod
    def solver(self):
        pass

    @abstractmethod
    def vars(self):
        '''
        iterator over all solution variables
        '''

    def z3_to_py(self, v, x):
        if isinstance(x, IntNumRef):
            return x.as_long()
        if isinstance(x, BoolRef):
            return bool(x)
        if x is None:
            if isinstance(v, BoolRef):
                return None
            raise TypeError(f'unconstrained non-boolean variable {v}')
        raise TypeError(f"can't handle object {repr(x)}")

    @abstractmethod
    def model_to_layers(self, model):
        '''
        return (solution layers, solution extra)
        '''

    def solutions(self):
        '''
        iterator over all solutions.

        undefined behavior if multiple solution iterators are created.
        '''
        s = self.solver
        V = list(self.vars())
        while s.check() == sat:
            m = s.model()
            vals = [self.z3_to_py(v, m[v]) for v in V]
            for ml in itertools.product(*([False, True] if val is None else [val] for val in vals)):
                m2 = {k: v for (k, v) in zip(V, ml)}
                solution = self.model_to_layers(m2)
                yield solution
            s.add(Or([v != val for v, val in zip(V, vals) if val is not None]))
