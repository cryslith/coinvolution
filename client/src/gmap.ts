// Manually translated from gmap.py

type alphalist = [number[], number?];

export function* unique_by_orbit(l: Iterable<Dart>, a: alphalist) {
  const seen = new Set();
  for (const dart of l) {
    if (seen.has(dart)) {
      continue;
    }
    yield dart;
    for (const n of dart.orbit(a)) {
      seen.add(n);
    }
  }
}

export function cell_alphas(i: number, dim?: number): alphalist {
  if (dim === undefined) {
    return [[...Array(i).keys()], i + 1];
  } else {
    return [[...Array(dim + 1).keys()].filter(j => j != i), undefined];
  }
}

export class Dart {
  public alpha: this[];
  public number: number;

  constructor(dimension: number, number: number) {
    this.alpha = Array.from({length: dimension + 1}, _ => this);
    this.number = number;
  }

  al(...ii: number[]) {
    let d = this;
    for (const i of ii) {
      d = d.alpha[i];
    }
    return d;
  }

  _alphas(a: alphalist): number[] {
    let [alphas, d] = a;
    if (d === undefined) {
      return alphas;
    } else {
      return [...alphas, ...[...Array(this.alpha.length - d).keys()].map(x => x + d!)];
    }
  }

  *orbit_paths(a: alphalist): IterableIterator<[number[], this]> {
    const seen = new Set();
    const frontier: [number[], this][] = [[[], this]];
    while (frontier.length) {
      const [path, dart] = frontier.shift()!;
      if (seen.has(dart)) {
        continue;
      }
      seen.add(dart);
      yield [path, dart];
      for (const i of this._alphas(a)) {
        const neighbor = dart.alpha[i];
        frontier.push([[...path, i], neighbor]);
      }
    }
  }

  *orbit(a: alphalist) {
    for (const [_, dart] of this.orbit_paths(a)) {
      yield dart;
    }
  }

  cell(i: number, dim?: number) {
    return this.orbit(cell_alphas(i, dim));
  }

  one_dart_per_incident_orbit(a: alphalist, b: alphalist) {
    return unique_by_orbit(this.orbit(b), a);
  }

  one_dart_per_incident_cell(i: number, j: number, dim?: number) {
    return this.one_dart_per_incident_orbit(cell_alphas(i, dim), cell_alphas(j, dim));
  }
}

export class GMap {
  public dimension: number;
  public darts: Dart[];

  constructor(dimension: number, darts?: Dart[]) {
    this.dimension = dimension;
    if (darts === undefined) {
      this.darts = [];
    } else {
      this.darts = [...darts];
    }
  }

  one_dart_per_orbit(a: alphalist) {
    return unique_by_orbit(this.darts, a);
  }

  one_dart_per_cell(i: number, dim?: number) {
    return this.one_dart_per_orbit(cell_alphas(i, dim));
  }
}

export class OrbitMap<T> {
  private darts: Map<Dart, T>;
  public a: alphalist;

  constructor(a: alphalist) {
    this.darts = new Map();
    this.a = a;
  }

  get(dart: Dart) {
    return this.darts.get(dart);
  }

  set(dart: Dart, value: T) {
    for (const d of dart.orbit(this.a)) {
      this.darts.set(d, value);
    }
  }

  delete(dart: Dart) {
    for (const d of dart.orbit(this.a)) {
      this.darts.delete(d);
    }
  }
}

export class CellMap<T> extends OrbitMap<T> {
  constructor(i: number, dim?: number) {
    super(cell_alphas(i, dim));
  }
}
