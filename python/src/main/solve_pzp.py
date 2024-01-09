import argparse
import sys

from ..solvers import PZP_SOLVERS
from ..pzp import decode

def solutions(s):
    (variety, g, layers, extra) = decode(s)
    solver = PZP_SOLVERS[variety](g, layers, extra)
    return solver.solutions()

def main():
    p = argparse.ArgumentParser(description='Solve a puzzle from a puzz.link url.', epilog='Supported varieties:\n' + '\n'.join(PZP_SOLVERS.keys()))
    p.add_argument('url', help='puzz.link url')
    args = p.parse_args()

    (variety, g, layers, extra) = decode(args.url)
    print(f'{g.width}*{g.height} {variety}')
    solver = PZP_SOLVERS[variety](g, layers, extra)

    for (sol_layers, sol_extra) in solver.solutions():
        print(sol_layers, sol_extra)
