import {Vertex, Edge, Face} from "./graph.js";

export class GridDisplay {
  constructor(svg, n, m) {
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
        const f = new Face(
          facegroup,
          [[j, i], [j + 1, i], [j + 1, i + 1], [j, i + 1]],
          [j + 1/2, i + 1/2],
        );
        row.push(f);
      }
      faces.push(row);
    }

    for (let i = 0; i <= n; i++) {
      const row = [];
      for (let j = 0; j < m; j++) {
        const e = new Edge(
          edgegroup,
          [[j, i], [j + 1, i]],
          [[j, i], [j + 1/2, i - 1/2], [j + 1, i], [j + 1/2, i + 1/2]],
          [j + 1/2, i],
        );
        row.push(e);
      }
      edges.push(row);
    }

    for (let i = 0; i < n; i++) {
      const row = [];
      for (let j = 0; j <= m; j++) {
        const e = new Edge(
          edgegroup,
          [[j, i], [j, i + 1]],
          [[j, i], [j + 1/2, i + 1/2], [j, i + 1], [j - 1/2, i + 1/2]],
          [j, i + 1/2],
        );
        row.push(e);
      }
      edges.push(row);
    }

    for (let i = 0; i <= n; i++) {
      const row = [];
      for (let j = 0; j <= m; j++) {
        const v = new Vertex(
          vertgroup,
          [j, i],
          [[j - 1/2, i - 1/2], [j + 1/2, i - 1/2], [j + 1/2, i + 1/2], [j - 1/2, i + 1/2]],
        );
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
  return new GridDisplay(svg, 10, 20);
}
