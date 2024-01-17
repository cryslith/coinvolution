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
        return (solution layers, solution extra) from model.  model may be incomplete, indicating unknown variables.
        '''

    def solutions(self):
        '''
        iterator over all solutions.

        undefined behavior if multiple solution iterators are created.
        sets self.forced_variables when all solutions are fully consumed
        '''
        s = self.solver
        V = list(self.vars())
        forced_vars = None
        while s.check() == sat:
            m = s.model()
            m_ = {v: self.z3_to_py(v, m[v]) for v in V}
            if forced_vars is None:
                forced_vars = {v: x for v, x in m_.items() if x is not None}
            else:
                f = list(forced_vars)
                for v in f:
                    if forced_vars[v] != m_[v]:
                        del forced_vars[v]                
            val_list = [m_[v] for v in V]
            for ml in itertools.product(*([False, True] if x is None else [x] for x in val_list)):
                m2 = {v: x for (v, x) in zip(V, ml)}
                solution = self.model_to_layers(m2)
                yield solution
            s.add(Or([v != x for v, x in m_.items() if x is not None]))
        self.forced_variables = self.model_to_layers(forced_vars)

    def _forced_variables_slow(self):
        '''
        find all variables whose values are forced by the constraints
        '''
        # in practice this implementation seems to be way slower than just enumerating all solutions.
        # can be improved by:
        # - adding persistent constraints for known-forced variables
        # - detecting early when a variable is not forced because of a different solution
        s = self.solver
        if s.check() != sat:
            return unsat
        m = s.model()
        m2 = {v: self.z3_to_py(v, m[v]) for v in self.vars()}
        forced_vars = {}
        for v in self.vars():
            if m2[v] is None:
                continue
            s.push()
            s.add(v != m2[v])
            forced = s.check() == unsat
            s.pop()
            if forced:
                forced_vars[v] = m2[v]
        return self.model_to_layers(forced_vars)
