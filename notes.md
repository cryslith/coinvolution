## GMap library

- Add unit tests
- Optimize data structures
  - Consider using tinyset or smallset for orbit searches
  - Check low dimension (<= 2) for orbit searches
  - Avoid allocating paths in orbit searches
- Add more constructions:
  - prisms (product with an interval)
    - redo grids in terms of prisms?
  - coproduct (should be easy)

## Graph display

Goals:
- Interaction with grids should be as natural as possible
- Support many ways of displaying information over graphs (e.g. color, text, arrows on edges, ...)
- Support generalized cells (directed edges, single darts, ...) not just 0/1/2-cells
- Allow graphs to contain gluings (e.g. tori) naturally
- Stretch goal: allow the user to live-edit the graph itself

Non-goals
- 3D display of any kind
- Support for dimension higher than 2

Plan:
- Use face polygons to narrow down clicks to a single face
- Within a face, use the Voronoi diagram of the half-edges to determine which dart was clicked

Other thoughts:
- Lots of examples of real puzzles solved with it
  - e.g. every nikoli puzzle
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

## Building & Testing

    cd coin-web
    wasm-pack build --target web -- --features console_error_panic_hook
    python3 -m http.server

