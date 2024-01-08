from .gmap import GMap

class Grid(GMap):
    '''
    n * m grid; n rows, m columns.
    rows increase from north to south,
    columns increase from west to east

    some squares may be deleted.
    '''
    def __init__(self, n, m):
        super().__init__(2)
        rows = []
        for _ in range(n):
            row = []
            for _ in range(m):
                row.append(self.make_polygon(4))
            for s0, s1 in zip(row, row[1:]):
                s0.al(0, 1).sew(2, s1.al(1))
            rows.append(row)
        for r0, r1 in zip(rows, rows[1:]):
            for s0, s1 in zip(r0, r1):
                s0.al(1, 0, 1).sew(2, s1)
        # Each square is the dart on the square's north edge, northwest vertex
        self.squares = rows

    def vertex_grid(self):
        vrows = []
        for row in self.squares:
            vrow = []
            for d in row:
                vrow.append(d)
            vrow.append(d.al(0))
            vrows.append(vrow)
        lastvrow = []
        for d in row:
            lastvrow.append(d.al(1, 0, 1))
        lastvrow.append(d.al(1, 0, 1, 0))
        vrows.append(lastvrow)
        return vrows

    def vertex_positions(self):
        pos = CellDict(0)
        for i, row in enumerate(self.vertex_grid()):
            for j, d in enumerate(row):
                pos[d] = (j, i)
        return pos
