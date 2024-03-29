import argparse
import sys

from ..solvers import pzp_solvers
from ..pzp import decode
from ..draw.console import draw_grid

PZP_SOLVERS = pzp_solvers()

def solutions(s):
    (variety, g, layers, extra) = decode(s)
    solver = PZP_SOLVERS[variety](g, layers, extra)
    return (variety, g, layers, extra, solver.solutions())

def main():
    p = argparse.ArgumentParser(description='Solve a puzzle from a puzz.link url.', epilog='Supported varieties:\n' + '\n'.join(PZP_SOLVERS.keys()))
    p.add_argument('url', help='puzz.link url')
    p.add_argument('-a', '--ascii', action='store_true')
    p.add_argument('-q', '--hide-solutions', action='store_true')
    p.add_argument('-f', '--hide-forced', action='store_true')
    args = p.parse_args()

    (variety, g, layers, extra) = decode(args.url)
    print(f'{g.width}*{g.height} {variety}')
    solver = PZP_SOLVERS[variety](g, layers, extra)

    i = 0
    for (sol_layers, sol_extra) in solver.solutions():
        if not args.hide_solutions:
            print(f'solution {i}:')
            print(draw_grid(g, layers + sol_layers, args.ascii))
            if sol_extra is not None:
                print(sol_extra)
            print()
        i += 1
    print(f'total {i}')

    if i > 1 and not args.hide_forced:
        print('forced variables:')
        (sol_layers, sol_extra) = solver.forced_variables
        print(draw_grid(g, layers + sol_layers, args.ascii))
        if sol_extra is not None:
            print(sol_extra)
