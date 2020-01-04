#!/usr/bin/env python3

from inspect import cleandoc
from gmap import GMap
import sys

def dump(gmap):
    output = []
    for d in sorted(gmap.darts, key=lambda d: d.number):
        output.append('{}: {}'.format(
                d, ' '.join(str(x) for x in d.alpha)))
    return '\n'.join(output)

def graphvizify(gmap):
    # give each vertex a unique number
    i = 0
    vertices = {}
    for vertex in gmap.all_cells(0):
        for d in vertex:
            vertices[d] = i
        i += 1

    output = []
    for edge in gmap.one_dart_per_cell(1):
        (v0, v1) = edge, edge.al(0)
        output.append('  v{} -> v{};'.format(
                vertices[v0], vertices[v1]))
    return cleandoc('''digraph foo {{
                         node[shape=point];
                         edge[dir=none];
                       {}
                       }}''').format('\n'.join(output))

def demo_gmap():
    g = GMap(2)
    g.make_cube()
    g.check_validity()
    print(graphvizify(g))

if __name__ == '__main__':
    demo_gmap()
