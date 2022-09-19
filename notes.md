## GMap library

- Optimize data structures
  - Consider using tinyset or smallset for orbit searches
- Add more constructions:
  - prisms (product with an interval)
    - redo grids in terms of prisms?
  - coproduct (should be easy)

## Graph display

Todo:
- solver integration
- graph editor
- test on more grids
  - Fix bug with selecting lines on thin triangles
- layer options (marker types, size/color config)
- zoom and pan
- display selected cell for text
- deselect cell
- Diagonals (e.g. Gokigen-Naname)
  - How does this generalize to non-quadrilaterals?
- Exclusive directed markers
- Click and drag
- Custom SVG markers?


Goals:
- Interaction with grids should be as natural as possible
- Support many ways of displaying information over graphs (e.g. color, text, arrows on edges, ...)
- Support generalized cells (directed edges, single darts, ...) not just 0/1/2-cells
- Allow graphs to contain gluings (e.g. tori) naturally
  - Better handled on solver side
- Allow the user to live-edit the graph itself?

Non-goals
- 3D display of any kind
- Support for dimension higher than 2

## Solving

- Unsat cores

Tricky things:
- Connectivity constraints (e.g. Slitherlink)
  - Build a minimum-distance tree over included nodes
- Sightlines (e.g. Akari, Kakuro)
  - Use coordinates?
  - Specify alpha indices?
- Shapes (e.g. Statue Garden, LITS)
- Custom extra info
  - Can always send as a user-specified layer

Other thoughts:
- Build solvers for puzzle genres
  - Nikoli & others
- Build demos for existing puzzles
  - Mobius strip: https://puzzles.mit.edu/2021/puzzle/slithe%C9%B9l%E1%B4%89u%CA%9Es/
  - Triangle: https://puzzles.mit.edu/2022/puzzle/dancing-triangles/
  - Hex: https://puzzles.covering.space/58/ https://puzzles.covering.space/62/ https://puzzles.covering.space/54/
  - Torus: https://puzzles.covering.space/3/
- Find efficient ways of using SAT solvers
- Consider using ASP for solving

Related work:
https://www.mstang.xyz/noq - https://github.com/mstang107/noq (using ASP!)
https://github.com/tomvbussel/fillomino
https://github.com/obijywk/grilops
https://github.com/jenna-h/hunt-logic
https://github.com/semiexp/cspuz

Resources:
https://theory.stanford.edu/~nikolaj/programmingz3.html
https://arxiv.org/abs/1708.01745
https://sat-smt.codes/SAT_SMT_by_example.pdf
https://rise4fun.com/z3/tutorial
https://cs.stackexchange.com/questions/111410/sat-algorithm-for-determining-if-a-graph-is-disjoint

## Building & Testing

    cd coin-web
    wasm-pack build --debug --target web -- --features console_error_panic_hook
    python3 -m http.server

