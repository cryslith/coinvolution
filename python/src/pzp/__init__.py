from ..gmap import Alphas
from ..gmap.grid import Grid
from ..puzzle import Layer

import re

def decode(s):
    m = re.match(r'(.*?p\?)?(.*?)/(.*?)/(.*?)/(.*)', s)
    if m is None:
        raise ValueError
    variety = m.group(2)
    width = int(m.group(3))
    height = int(m.group(4))
    data = m.group(5)
    g = Grid(height, width)
    layers = []
    if variety == 'slither':
        decoded = decode4Cell(height, width, data)
        layers = [
            Layer(
                'clues',
                Alphas.FACE,
                {g[y, x]: v for (y, r) in enumerate(decoded) for (x, v) in enumerate(r) if v >= 0},
            ),
        ]
    else:
        raise ValueError(f'unknown variety {variety}')
    return (variety, g, layers)

def decode4Cell(height, width, data):
    r = []
    for c in data:
        if '0' <= c <= '4':
            r.append(int(c, 16))
            continue
        if '5' <= c <= '9':
            r.append(int(c, 16) - 5)
            r.append(-1)
            continue
        if 'a' <= c <= 'e':
            r.append(int(c, 16) - 10)
            r.extend([-1, -1])
            continue
        if 'g' <= c <= 'z':
            r.extend([-1] * (int(c, 36) - 15))
            continue
        if c == '.':
            r.append(-2)
            continue
        raise ValueError(f'unrecognized character {c}')

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[i:i+width] for i in range(0, width*height, width)]
