import {Dart, GMap, CellMap} from "./gmap.js";
import {deser_gmap, deser_cellmap} from "./ser.js";

const WIDTH = 1000, HEIGHT = 1000;
const SCALE = 40;
const OFFSET = 50;

SVG.on(document, 'DOMContentLoaded', () => {
  const draw = SVG().addTo('body').size(WIDTH, HEIGHT);

  example_gmap(draw);
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
      this.elem.css('pointer-events', '');
    } else {
      this.elem.css('pointer-events', 'none');
    }
  }

  color(baseColor, change) {
    let c = new SVG.Color(baseColor).hsl();
    if (this.hovered) {
      if (c.l > 0x80) {
        c.l -= change;
      } else if (c.l > 0x80) {
        c.l += change;
      } else {
        c.l += change;
      }
    }
    return c;
  }
}

class Vertex extends Cell {
  constructor(svg, position) {
    super(svg);
    this.position = position;
    this.update();
  }

  createElem(svg) {
    return svg.circle();
  }

  update() {
    super.update();
    this.elem
      .size(4)
      .center(...this.position)
      .fill({color: this.color("#222", 0x20)});
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
    return svg.line();
  }

  update() {
    super.update();
    this.elem
      .attr({
        x1: this.ends[0][0],
        y1: this.ends[0][1],
        x2: this.ends[1][0],
        y2: this.ends[1][1],
      })
      .stroke({color: this.color('#222', 0x10), width: 2});
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

class GMapDisplay {
  constructor(svg, gmap, vertex_positions) {
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
      f.setActive(true);
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

  new GMapDisplay(svg, gmap, vertex_positions);
}
