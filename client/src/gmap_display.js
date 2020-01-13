import {Vertex, Edge, Face} from "./graph.js";
import {Dart, GMap, CellMap} from "./gmap.js";
import {deser_gmap, deser_cellmap} from "./ser.js";

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

export class GMapDisplay {
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


export async function example_gmap(svg) {
  const example_grid = await fetch('example_grid.json').then(x => x.json());
  const gmap = deser_gmap(example_grid);
  const vertex_positions = deser_cellmap(gmap, example_grid["vertex_positions"], x => x.map(y => y * SCALE + OFFSET));

  return new GMapDisplay(svg, gmap, vertex_positions);
}
