import {Vertex, Edge, Face} from "./graph.js";

export class ColorVertex extends Vertex {
  constructor(svg, position, bounds, colorMap) {
    super(svg, position, bounds);
    this.colorMap = colorMap;
  }

  color() {
    return this.colorMap.get(this.puzzleState.coloring) || super.color();
  }

  radius() {
    if (this.puzzleState.coloring) {
      return 4 * super.radius();
    }
    return super.radius();
  }

  click() {
    const keys = [...this.colorMap.keys()];
    this.puzzleState.coloring = keys[(keys.indexOf(this.puzzleState.coloring) + 1) % keys.length];
    this.update();
  }
}

export class ColorEdge extends Edge {
  constructor(svg, ends, bounds, center, colorMap) {
    super(svg, ends, bounds, center);
    this.colorMap = colorMap;
  }

  color() {
    return this.colorMap.get(this.puzzleState.coloring) || super.color();
  }

  width() {
    if (this.puzzleState.coloring) {
      return 2 * super.width();
    }
    return super.width();
  }

  click() {
    const keys = [...this.colorMap.keys()];
    this.puzzleState.coloring = keys[(keys.indexOf(this.puzzleState.coloring) + 1) % keys.length];
    this.update();
  }
}

export class ColorFace extends Face {
  constructor(svg, points, center, colorMap) {
    super(svg, ends, points, center);
    this.colorMap = colorMap;
  }

  color() {
    return this.colorMap.get(this.puzzleState.coloring) || super.color();
  }

  click() {
    const keys = [...this.colorMap.keys()];
    this.puzzleState.coloring = keys[(keys.indexOf(this.puzzleState.coloring) + 1) % keys.length];
    this.update();
  }
}
