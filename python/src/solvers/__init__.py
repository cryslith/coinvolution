from abc import ABC, abstractmethod

from . import simpleloop

from z3 import Or, sat

PZP_SOLVERS = {
    'simpleloop': simpleloop.solve,
}

class Solver(ABC):
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
            nsols += 1
            yield solution
            s.add(Or([v != m[v] for v in self.vars()]))
