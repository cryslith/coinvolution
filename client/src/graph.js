export class Cell {
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
      .mouseover(this.mouseover.bind(this))
      .mouseout(this.mouseout.bind(this))
      .click(this.click.bind(this));
  }

  mouseover() {
    this.hovered = true;
    this.update();
  }

  mouseout() {
    this.hovered = false;
    this.update();
  }

  click() {
  }

  update() {
    if (this.active) {
      this.elem.css('pointer-events', 'fill');
    } else {
      this.elem.css('pointer-events', 'none');
    }
  }

  hoverColor(baseColor, changeDown, changeUp) {
    if (changeUp === undefined) {
      changeUp = 2 * changeDown;
    }
    let c = new SVG.Color(baseColor).hsl();
    if (this.hovered) {
      c.s = 100;
      if (c.l >= 50) {
        c.l -= changeDown;
      } else {
        c.l += changeUp;
      }
    }
    return c;
  }
}

export class Vertex extends Cell {
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
      .size(this.radius())
      .center(...this.position)
      .fill({color: this.hoverColor(this.color(), 25)});
    this.cover.plot(this.bounds)
      .fill('none');
  }

  color() {
    return '#222';
  }

  radius() {
    return 1/10;
  }
}

export class Edge extends Cell {
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
      .stroke({color: this.hoverColor(this.color(), 25), width: this.width()});
    this.cover.plot(this.bounds)
      .fill('none');
  }

  color() {
    return '#222';
  }

  width() {
    return 1/20;
  }
}

export class Face extends Cell {
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
      .fill({color: this.hoverColor(this.color(), 8)});
  }

  color() {
    return '#ececec';
  }
}
