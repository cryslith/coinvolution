#!/usr/bin/env python3

from inspect import cleandoc
from gmap import GMap

def graphvizify(gmap):
    # give each vertex a unique number
    i = 0
    vertices = {}
    for vertex in gmap.all_cells(0):
        for d in vertex:
            vertices[d] = i
        i += 1

    output = []
    for edge in gmap.all_cells(1):
        (v0, v1) = edge
        output.append('  {} -- {};'.format(vertices[v0], vertices[v1]))
    return cleandoc('''graph foo {{
                         node[shape=point];
                       {}
                       }}''').format('\n'.join(output))

def demo_gmap():
    g = GMap(2)
    g.make_polygon(10)
    g.check_validity()
    print(graphvizify(g))

if __name__ == '__main__':
    demo_gmap()
