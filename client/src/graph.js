import {Dart, GMap, CellMap} from "./gmap.js";
import {deser_gmap, deser_cellmap} from "./ser.js";

const WIDTH = 1000, HEIGHT = 500;

SVG.on(document, 'DOMContentLoaded', () => {
  const draw = SVG('#draw').size(WIDTH, HEIGHT);
  const puzzle = draw.group().scale(40).translate(50, 50);

  const grid = example_grid(puzzle);
  for (const [i, b] of ['vbutton', 'ebutton', 'fbutton'].entries()) {
    const button = document.getElementById(b);
    SVG.on(button, 'click', () => grid.setActive(i));
  }
});

class Cell {
  constructor(svg) {
    this.puzzleState = {};
    this.hovered = false;
    this.active = false;
    this.elem = this.createElem(svg);
    this.initInputs();
  }

  setActive(active) {
    this.active = active;
    if (!active) {
      this.hovered = false;
    }
    this.update();
  }

  initInputs() {
    this.elem
      .mouseover(() => {
        this.hovered = true;
        this.update();
      })
      .mouseout(() => {
        this.hovered = false;
        this.update();
      });
  }

  update() {
    if (this.active) {
      this.elem.css('pointer-events', 'fill');
    } else {
      this.elem.css('pointer-events', 'none');
    }
  }

  color(baseColor, changeDown, changeUp) {
    if (changeUp === undefined) {
      changeUp = 2 * changeDown;
    }
    let c = new SVG.Color(baseColor).hsl();
    if (this.hovered) {
      if (c.l >= 50) {
        c.l -= changeDown;
      } else {
        c.l += changeUp;
      }
    }
    return c;
  }
}

class Vertex extends Cell {
  constructor(svg, position, bounds) {
    super(svg);
    this.position = position;
    this.bounds = bounds;
    this.update();
  }

  createElem(svg) {
    const g = svg.group();
    this.circle = g.circle();
    this.cover = g.polygon();
    return g;
  }

  update() {
    super.update();
    this.circle
      .size(1/10)
      .center(...this.position)
      .fill({color: this.color("#222", 25)});
    this.cover.plot(this.bounds)
      .fill('none');
  }
}

class Edge extends Cell {
  constructor(svg, ends, bounds, center) {
    super(svg);
    this.ends = ends;
    this.bounds = bounds;
    this.center = center;
    this.update();
  }

  createElem(svg) {
    const g = svg.group();
    this.line = g.line();
    this.cover = g.polygon();
    return g;
  }

  update() {
    super.update();
    this.line
      .attr({
        x1: this.ends[0][0],
        y1: this.ends[0][1],
        x2: this.ends[1][0],
        y2: this.ends[1][1],
      })
      .stroke({color: this.color('#222', 25), width: 1/20});
    this.cover.plot(this.bounds)
      .fill('none');
  }
}

class Face extends Cell {
  constructor(svg, points, center) {
    super(svg);
    this.points = points;
    this.center = center;
    this.update();
  }

  createElem(svg) {
    return svg.polygon();
  }

  update() {
    super.update();
    this.elem
      .plot(this.points)
      .fill({color: this.color('#edc35a', 8)});
  }
}

class Graph {
  constructor(svg, vertices, edges, faces) {
    this.vertices = vertices;
    this.edges = edges;
    this.faces = faces;
  }
}

function average2(points) {
  return points
    .reduce(([x0, y0], [x1, y1]) => [x0 + x1, y0 + y1])
    .map(x => x / points.length);
}

function sort_positions(points, center) {
  const [cx, cy] = center ? center : average2(points);
  const angles = points.map(([x, y]) => [Math.atan2(x - cx, y - cy), [x, y]]);
  return angles
    .sort(([a0, _0], [a1, _1]) => a0 - a1)
    .map(([_, p]) => p);
}

function* imap(f, l) {
  for (const x of l) {
    yield f(x);
  }
}

class GridDisplay {
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

class GMapDisplay {
  constructor(svg, gmap, vertex_positions) {
    // TODO need to implement bounds, setActive - see GridDisplay
    this.gmap = gmap;
    this.vertex_positions = vertex_positions;
    this.vertgroup = svg.group();
    this.edgegroup = svg.group().insertBefore(this.vertgroup);
    this.facegroup = svg.group().insertBefore(this.edgegroup);
    this.vertices = new CellMap(0);
    this.edges = new CellMap(1);
    this.faces = new CellMap(2);

    for (const face of gmap.one_dart_per_cell(2)) {
      let points =
          [...face.one_dart_per_incident_cell(0, 2)]
          .map(v => vertex_positions.get(v));
      const center = average2(points);
      points = sort_positions(points, center);
      const f = new Face(
        this.facegroup,
        points,
        center,
      );
      this.faces.set(face, f);
    }

    for (const edge of gmap.one_dart_per_cell(1)) {
      const ends =
            [...edge.one_dart_per_incident_cell(0, 1)]
            .map(v => vertex_positions.get(v));
      const center = average2(ends);
      const bounds = sort_positions(
        [...ends,
         ...imap((d => this.faces.get(d).center),
                 edge.one_dart_per_incident_cell(2, 1))],
        center);
      const e = new Edge(
        this.edgegroup,
        ends,
        bounds,
        center,
      );
      this.edges.set(edge, e);
    }

    for (const vertex of gmap.one_dart_per_cell(0)) {
      const v = new Vertex(this.vertgroup, vertex_positions.get(vertex));
      this.vertices.set(vertex, v);
    }
  }
}

async function example_gmap(svg) {
  const example_grid = await fetch('example_grid.json').then(x => x.json());
  const gmap = deser_gmap(example_grid);
  const vertex_positions = deser_cellmap(gmap, example_grid["vertex_positions"], x => x.map(y => y * SCALE + OFFSET));

  return new GMapDisplay(svg, gmap, vertex_positions);
}

function example_grid(svg) {
  return new GridDisplay(svg, 10, 20);
}
