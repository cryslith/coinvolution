import {example_grid} from "./grid_display.js";

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
