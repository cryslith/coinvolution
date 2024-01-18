from ..gmap import Alphas
from ..gmap.grid import Grid
from ..puzzle import Layer, Display

import re
from itertools import chain

def decode(s):
    m = re.match(r'(.*?p\?)?(.*?)/(.*?)/(.*?)/(.*)', s) # todo change this to url decoding, strip off trailing slash
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
                {g[y, x]: v for (y, r) in enumerate(decoded) for (x, v) in enumerate(r) if v is not None and v != '?'},
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
    elif variety == 'yajilin':
        (decoded, data) = decode_arrow_number_16(height, width, data)
        if data:
            raise ValueError('extra data')
        data = {g[y, x]: v for (y, r) in enumerate(decoded) for (x, v) in enumerate(r) if v is not None}
        clues = {f: n for f, (d, n) in data.items() if d == 0}
        clues_arrow = {fe_arrow(y, x, d): n for (y, x, _), (d, n) in data.items() if d != 0}
        layers = [
            Layer(
                'clues',
                Alphas.FACE,
                clues,
                Display.text,
            ),
            Layer(
                'clues_arrow',
                Alphas.SIDE,
                clues_arrow,
                # todo composite display
            ),
        ]
    elif variety == 'numlin':
        (decoded, data) = decode_number_16(height, width, data)
        if data:
            raise ValueError('extra data')
        layers = [
            Layer(
                'clues',
                Alphas.FACE,
                {g[y, x]: v for (y, r) in enumerate(decoded) for (x, v) in enumerate(r) if v is not None},
                Display.text,
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
            r.append(None)
            continue
        if 'a' <= c <= 'e':
            r.append(int(c, 16) - 10)
            r.extend([None, None])
            continue
        if 'g' <= c <= 'z':
            r.extend([None] * (int(c, 36) - 15))
            continue
        if c == '.':
            r.append('?')
            continue
        raise ValueError(f'unrecognized character {c}')
    else:
        rem += 1

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[i:i+width] for i in range(0, width*height, width)], data[rem:]

def read_number_16(data):
    c = data[0]
    if '0' <= c <= '9' or 'a' <= c <= 'f':
        return int(c, 16), 1
    if c == '-':
        return int(data[1:3], 16), 3
    if c == '+':
        return int(data[1:4], 16), 4
    if c == '=':
        return int(data[1:4], 16) + 4096, 4
    if c in '%@':
        return int(data[1:4], 16) + 8192, 4
    if c == '*':
        return int(data[1:5], 16) + 12240, 5
    if c == '$':
        return int(data[1:6], 16) + 77776, 6
    if c == '.':
        return '?', 1
    raise ValueError

def decode_number_16(height, width, data):
    r = []
    i = 0
    while len(r) < width * height:
        if 'g' <= data[0] <= 'z':
            r.extend([None] * (int(data[0], 36) - 15))
            data = data[1:]
            continue
        d, n = read_number_16(data)
        r.append(d)
        data = data[n:]

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[i:i+width] for i in range(0, width*height, width)], data

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

def decode_arrow_number_16(height, width, data):
    r = []
    skip = 0
    for (i, c) in enumerate(data):
        if skip > 0:
            skip -= 1
            continue
        if len(r) >= width * height:
            break
        if '0' <= c <= '4':
            direction = int(c, 16)
            num = None if data[i+1] == '.' else int(data[i+1], 16)
            skip = 1
            r.append((direction, num))
            continue
        if '5' <= c <= '9':
            direction = int(c, 16) - 5
            num = int(data[i+1:i+2], 16)
            skip = 2
            r.append((direction, num))
            continue
        if 'a' <= c <= 'z':
            r.extend([None] * (int(c, 36) - 9))
            continue
        raise ValueError(f'unrecognized character {c}')
    else:
        i += 1

    if len(r) != width * height:
        raise ValueError(f"decoded length {len(r)} doesn't match dimensions {width=} {height=}")

    return [r[j:j+width] for j in range(0, width*height, width)], data[i:]

def fe_arrow(y, x, direction):
    if direction == 1: # up
        return (y, x, 0)
    if direction == 2: # down
        return (y, x, 4)
    if direction == 3: # left
        return (y, x, 6)
    if direction == 4: # right
        return (y, x, 2)
    raise ValueError(f'invalid {direction=}')
