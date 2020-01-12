// Manually translated from gmap.py

export function* group_by_cell(l: Iterable<Dart>, i: number, dim?: number) {
  const seen = new Set();
  for (const dart of l) {
    if (seen.has(dart)) {
      continue;
    }
    const cell = [];
    for (const n of dart.cell(i, dim)) {
      cell.push(n);
      seen.add(n);
    }
    yield cell;
  }
}

export function* unique_by_cell(l: Iterable<Dart>, i: number, dim?: number) {
  const seen = new Set();
  for (const dart of l) {
    if (seen.has(dart)) {
      continue;
    }
    yield dart;
    for (const n of dart.cell(i, dim)) {
      seen.add(n);
    }
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

  *orbit_paths(alphas: number[]): IterableIterator<[number[], this]> {
    const seen = new Set();
    const frontier: [number[], this][] = [[[], this]];
    while (frontier.length) {
      const [path, dart] = frontier.shift()!;
      if (seen.has(dart)) {
        continue;
      }
      seen.add(dart);
      yield [path, dart];
      for (const i of alphas) {
        const neighbor = dart.alpha[i];
        frontier.push([[...path, i], neighbor]);
      }
    }
  }

  *orbit(alphas: number[]) {
    for (const [_, dart] of this.orbit_paths(alphas)) {
      yield dart;
    }
  }

  cell_paths(i: number, dim?: number) {
    if (dim === undefined) {
      dim = this.alpha.length - 1
    }
    const alphas = [...Array(dim + 1).keys()].filter(j => j != i);
    return this.orbit_paths(alphas);
  }

  *cell(i: number, dim?: number) {
    for (const [_, dart] of this.cell_paths(i, dim)) {
      yield dart;
    }
  }

  one_dart_per_incident_cell(i: number, j: number, dim?: number) {
    return unique_by_cell(this.cell(j, dim), i, dim);
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

  one_dart_per_cell(i: number, dim?: number) {
    return unique_by_cell(this.darts, i, dim);
  }

  all_cells(i: number, dim?: number) {
    return group_by_cell(this.darts, i, dim);
  }
}

export class CellMap<T> {
  private darts: Map<Dart, T>;
  public i: number;
  public dim?: number;

  constructor(i: number, dim?: number) {
    this.darts = new Map();
    this.i = i;
    this.dim = dim;
  }

  get(dart: Dart) {
    return this.darts.get(dart);
  }

  set(dart: Dart, value: T) {
    for (const d of dart.cell(this.i, this.dim)) {
      this.darts.set(d, value);
    }
  }

  delete(dart: Dart) {
    for (const d of dart.cell(this.i, this.dim)) {
      this.darts.delete(d);
    }
  }
}
