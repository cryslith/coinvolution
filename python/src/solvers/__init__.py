from abc import ABC, abstractmethod

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

    def z3_to_py(self, x):
        if isinstance(x, IntNumRef):
            return x.as_long()
        if isinstance(x, BoolRef):
            return bool(x)
        raise TypeError

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
        while s.check() == sat:
            m = s.model()
            m2 = {v: self.z3_to_py(m[v]) for v in self.vars()}
            solution = self.model_to_layers(m2)
            yield solution
            s.add(Or([v != m2[v] for v in self.vars()]))
