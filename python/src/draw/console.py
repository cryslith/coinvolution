from ..gmap import Alphas
from ..puzzle import Display

BRANCH_SYM = ' ╵╶└╷│┌├╴┘─┴┐┤┬┼'

def draw_grid(g, layers, ascii=False):
    canvas = [[' '] * (2*g.width + 1) for _ in range(2*g.height + 1)]
    for y in range(g.height + 1):
        for x in range(g.width + 1):
            canvas[2 * y][2 * x] = '·'
    for l in layers:
        if l.display is None:
            continue
        if l.display == Display.text:
            if l.alphas == Alphas.FACE:
                for k, v in l.data.items():
                    (y, x) = g.f_loc(k)
                    v = str(v)[:1]
                    if not v:
                        continue
                    canvas[2*y+1][2*x+1] = v
                continue
            if l.alphas == Alphas.EDGE:
                for k, v in l.data.items():
                    (y, x) = g.e_loc2(k)
                    v = str(v)[:1]
                    if not v:
                        continue
                    canvas[y][x] = v
                continue
            if l.alphas == Alphas.VERTEX:
                for k, v in l.data.items():
                    (y, x) = g.v_loc(k)
                    v = str(v)[:1]
                    if not v:
                        continue
                    canvas[2*y][2*x] = v
                continue
            raise NotImplementedError
        if l.display == Display.line:
            if l.alphas == Alphas.EDGE:
                for k, v in l.data.items():
                    if not v:
                        continue
                    (y, x) = g.e_loc2(k)
                    canvas[y][x] = '│' if y%2 == 0 else '─'
                if not ascii:
                    for f in g.faces():
                        (y, x) = g.f_loc(f)
                        sym = BRANCH_SYM[sum(1 << i for i, e in enumerate([
                            g.e_top(y, x), 
                            g.e_right(y, x), 
                            g.e_bottom(y, x), 
                            g.e_left(y, x), 
                        ]) if l.data.get(e))]
                        if sym != ' ':
                            canvas[2*y+1][2*x+1] = sym
                continue
            raise NotImplementedError
        # if l.display == Display.edge:
        #     if l.alphas == Alphas.EDGE:
        #         for k, v in l.data.items():
        #             if not v:
        #                 continue
        #             (y, x) = g.e_loc2(k)
        #             canvas[y][x] = '-' if y%2 == 0 else '|'
        #         continue
        #     raise NotImplementedError
        if l.display == Display.surface:
            if l.alphas == Alphas.FACE:
                for k, v in l.data.items():
                    if not v:
                        continue
                    (y, x) = g.f_loc(k)
                    canvas[2*y+1][2*x+1] = '█'
                continue
            raise NotImplementedError
    output = '\n'.join(''.join(r) for r in canvas)
    if ascii:
        output = output.translate(str.maketrans('·█│─', '.X|-'))
    return output
