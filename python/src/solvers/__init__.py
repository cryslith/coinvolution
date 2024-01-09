from abc import ABC, abstractmethod

from z3 import Or, sat

def pzp_solvers():
    from . import simpleloop
    return {
        'simpleloop': simpleloop.S,
    }

class PSolver(ABC):
    @abstractmethod
    def __init__(self, g, layers, extra):
        pass

    @property
    @abstractmethod
    def solver(self):
        pass

    @abstractmethod
    def vars(self):
        '''
        iterator over all solution variables
        '''

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
            solution = self.model_to_layers(m)
            yield solution
            s.add(Or([v != m[v] for v in self.vars()]))
