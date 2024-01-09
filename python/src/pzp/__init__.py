from ..gmap import Alphas
from ..gmap.grid import Grid
from ..puzzle import Layer, Display

import re
from itertools import chain

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
    extra = None
    if variety == 'slither':
        (decoded, data) = decode_4cell(height, width, data)
        if data:
            raise ValueError('extra data')
        layers = [
            Layer(
                'clues',
                Alphas.FACE,
                {g[y, x]: v for (y, r) in enumerate(decoded) for (x, v) in enumerate(r) if v >= 0},
                Display.text,
            ),
        ]
    elif variety == 'simpleloop':
        (decoded, data) = decode_binary(height, width, data)
        if data:
            raise ValueError('extra data')
        layers = [
            Layer(
                'shaded',
                Alphas.FACE,
                {g[y, x]: int(v) for (y, r) in enumerate(decoded) for (x, v) in enumerate(r)},
                Display.surface,
            ),
        ]
    else:
        raise ValueError(f'unknown variety {variety}')
    return (variety, g, layers, extra)

def decode_4cell(height, width, data):
    r = []
    for (rem, c) in enumerate(data):
        if len(r) >= width * height:
            break
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
    else:
        rem += 1

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[i:i+width] for i in range(0, width*height, width)], data[rem:]

def decode_binary(height, width, data):
    r = []
    for (rem, c) in enumerate(data):
        if len(r) >= width * height:
            break
        n = int(c, 32)
        r.extend([((n >> i) & 1) == 1 for i in reversed(range(5))])
    else:
        rem += 1

    r = r[:width*height]

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[i:i+width] for i in range(0, width*height, width)], data[rem:]
