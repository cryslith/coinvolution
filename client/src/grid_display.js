import {Vertex, Edge, Face, FOREGROUND, BACKGROUND} from "./graph.js";
import {ColorEdge} from "./coloring.js";

export class GridDisplay {
  constructor(svg, n, m, mkVertex, mkEdge, mkFace) {
    this.n = n;
    this.m = m;

    const vertgroup = svg.group();
    const edgegroup = svg.group().insertBefore(vertgroup);
    const facegroup = svg.group().insertBefore(edgegroup);

    const vertices = [];
    const edges = [];
    const faces = [];

    for (let i = 0; i < n; i++) {
      const row = [];
      for (let j = 0; j < m; j++) {
        const f = mkFace(
          facegroup,
          [[j, i], [j + 1, i], [j + 1, i + 1], [j, i + 1]],
          [j + 1/2, i + 1/2],
        );
        f.update();
        row.push(f);
      }
      faces.push(row);
    }

    for (let i = 0; i <= n; i++) {
      const row = [];
      for (let j = 0; j < m; j++) {
        const e = mkEdge(
          edgegroup,
          [[j, i], [j + 1, i]],
          [[j, i], [j + 1/2, i - 1/2], [j + 1, i], [j + 1/2, i + 1/2]],
          [j + 1/2, i],
        );
        e.update();
        row.push(e);
      }
      edges.push(row);
    }

    for (let i = 0; i < n; i++) {
      const row = [];
      for (let j = 0; j <= m; j++) {
        const e = mkEdge(
          edgegroup,
          [[j, i], [j, i + 1]],
          [[j, i], [j + 1/2, i + 1/2], [j, i + 1], [j - 1/2, i + 1/2]],
          [j, i + 1/2],
        );
        e.update();
        row.push(e);
      }
      edges.push(row);
    }

    for (let i = 0; i <= n; i++) {
      const row = [];
      for (let j = 0; j <= m; j++) {
        const v = mkVertex(
          vertgroup,
          [j, i],
          [[j - 1/2, i - 1/2], [j + 1/2, i - 1/2], [j + 1/2, i + 1/2], [j - 1/2, i + 1/2]],
        );
        v.update();
        row.push(v);
      }
      vertices.push(row);
    }

    this.cells = [vertices, edges, faces];
    this.cellgroups = [vertgroup, edgegroup, facegroup];
  }

  setActive(i) {
    for (const [j, cs] of this.cells.entries()) {
      for (const c of cs.flat()) {
        c.setActive(i == j);
      }
    }
  }
}

export function example_grid(svg) {
  function mkColorEdge(svg, ends, bounds, center) {
    return new ColorEdge(svg, ends, bounds, center,
                         new Map([['on', FOREGROUND],
                                  ['off', BACKGROUND],
                                  [undefined, '#00f']]));
  }
  return new GridDisplay(svg, 10, 20, (...args) => new Vertex(...args), mkColorEdge, (...args) => new Face(...args));
}
