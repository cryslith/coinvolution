#!/usr/bin/env python3

from z3 import *
from pprint import pprint
from functools import reduce
from gmap import *
from inspect import cleandoc
from collections import Counter


WIDTH = 18
HEIGHT = 4

front = Grid(HEIGHT, WIDTH)
front.increase_dimension(3)
back = Grid(HEIGHT, WIDTH)
back.increase_dimension(3)

# red sew
e1 = front.squares[0][-1].al(0, 1)
e2 = back.squares[0][0].al(1)
for _ in range(HEIGHT):
    e1.sew(2, e2)
    e1 = e1.al(0, 1, 2, 1)
    e2 = e2.al(0, 1, 2, 1)

# blue sew
e1 = back.squares[0][-1].al(0, 1)
e2 = front.squares[0][0].al(1)
for _ in range(HEIGHT):
    e1.sew(2, e2)
    e1 = e1.al(0, 1, 2, 1)
    e2 = e2.al(0, 1, 2, 1)

# back-front sew
for fr, br in zip(front.squares, reversed(back.squares)):
    for fs, bs in zip(fr, br):
        fs.al(1, 0, 1).sew(3, bs)

combined = GMap(3, darts=(front.darts + back.darts))
combined.check_validity()


path = CellDict(1, 2)
pathOrder = CellDict(1, 2)
vertexOrder = CellDict(0, 2)
for e in combined.one_dart_per_cell(1, 2):
    path[e] = FreshBool()
    pathOrder[e] = FreshInt()
for v in combined.one_dart_per_cell(0, 2):
    vertexOrder[v] = FreshInt()

pathConstraints = []

for v in combined.one_dart_per_cell(0, 2):
    x = FreshInt()
    pathConstraints.append(x == Sum([
        IntSort().cast(path[edge])
        for edge in v.one_dart_per_incident_cell(1, 0, 2)
    ]))
    pathConstraints.append(Or(x == 2, x == 0))

    pathConstraints.append(If(
        Or([path[e] for e in v.one_dart_per_incident_cell(1, 0, 2)]),
        Or([And(path[e], pathOrder[e] == vertexOrder[v])
            for e in v.one_dart_per_incident_cell(1, 0, 2)]),
        vertexOrder[v] == -1,
    ))

for e in combined.one_dart_per_cell(1, 2):
    pathConstraints.append(If(path[e], pathOrder[e] >= 0, pathOrder[e] == -1))

    pathConstraints.append(Implies(
        pathOrder[e] > 0,
        And(
            Or([pathOrder[e] == 1 + vertexOrder[v]
                for v in e.one_dart_per_incident_cell(0, 1, 2)]),
            Or([Or(vertexOrder[v] == 0, pathOrder[e] == vertexOrder[v])
                for v in e.one_dart_per_incident_cell(0, 1, 2)]),
        )
    ))

pathConstraints.append(Sum([IntSort().cast(pathOrder[e] == 0)
                            for e in combined.one_dart_per_cell(1, 2)]) == 1)


slitherCount = CellDict(2, 2)
for s in combined.one_dart_per_cell(2, 2):
    slitherCount[s] = FreshInt()

slitherConstraints = []

for s in combined.one_dart_per_cell(2, 2):
    slitherConstraints.append(slitherCount[s] == Sum([
        IntSort().cast(path[edge])
        for edge in s.one_dart_per_incident_cell(1, 2, 2)
    ]))


FS = cleandoc('''
........33........
..................
.0..1.0...2.....0.
........0.....0...
''').split('\n')

BS = cleandoc('''
2.......1.0.....3.
....1....0....1...
...........1......
..1..0...2........
''').split('\n')

for S, side in [(FS, front), (BS, back)]:
    for sr, cr in zip(S, side.squares):
        for s, c in zip(sr, cr):
            if s != '.':
                s = int(s)
                slitherConstraints.append(s == slitherCount[c])


frontBackConstraints = []

S = cleandoc('''
.5..1.....35.2.1.0
45.1..4..3........
.......4...4.4...4
.23..313....22.1.6
''').split('\n')

for sr, cr in zip(S, front.squares):
    for s, c in zip(sr, cr):
        if s != '.':
            s = int(s)
            frontBackConstraints.append(s == slitherCount[c] + slitherCount[c.al(3)])


def solve():
    s = Solver()
    s.add(*pathConstraints)
    s.add(*slitherConstraints)
    s.add(*frontBackConstraints)
    while True:
        if s.check() != sat:
            print('unsat')
            return
        print('======================')
        m = s.model()

        vps = front.vertex_positions()

        for side in (front, back):
            er = side.squares[0][0]
            for i in range(HEIGHT):
                e = er
                for _ in range(WIDTH):
                    print('.-' if m[path[e]] else '. ', end='')
                    e = e.al(0, 1, 2, 1)
                print('.')
                e = er
                for _ in range(WIDTH):
                    print('| ' if m[path[e.al(1)]] else '  ', end='')
                    e = e.al(0, 1, 2, 1)
                print('|' if m[path[e.al(1)]] else ' ')
                er = er.al(1, 0, 1, 2)
            e = er
            for _ in range(WIDTH):
                print('.-' if m[path[e]] else '. ', end='')
                e = e.al(0, 1, 2, 1)
            print('.')
            print()
            print()

        s.add(Or([m[v] != v for v in path.values()]))


if __name__ == '__main__':
    solve()
