import {on_face_click} from "../pkg/coin_web.js";

export function make_face_clicker(state, face, vertex_locations) {
  let { svg, puzzle } = state;
  console.log(face, vertex_locations);
  let clicker = svg.polygon([].slice.call(vertex_locations)).fill('transparent').stroke({ width: 0.05, color: 'gray' });
  clicker.click(() => {
    on_face_click(puzzle, face);
  });
}
