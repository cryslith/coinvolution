import {Dart, GMap, CellMap} from "./gmap.js";

export interface SerGMap {
  dimension: number,
  darts: number[][],
}

export function deser_gmap(s: SerGMap): GMap {
  const darts = s.darts.map((_, i) => new Dart(s.dimension, i));
  for (const [i, d] of s.darts.entries()) {
    for (const [j, k] of d.entries()) {
      darts[i].alpha[j] = darts[k];
    }
  }
  return new GMap(s.dimension, darts);
}

export interface SerCellMap<T> {
  darts: {[i: string]: T}
  i: number,
  dim?: number,
}

export function deser_cellmap<T, U>(gmap: GMap, s: SerCellMap<T>, f: (t: T) => U) {
  const c = new CellMap<U>(s.i, s.dim);
  for (const [k, v] of Object.entries(s.darts)) {
    c.set(gmap.darts[parseInt(k)], f(v));
  }
  return c;
}
