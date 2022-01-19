import init, {initialize, set_panic_hook} from "../pkg/coin_web.js";

const WIDTH = 1000, HEIGHT = 500;

SVG.on(document, 'DOMContentLoaded', () => {
  init()
    .then(() => {
      set_panic_hook();
      
      const draw = SVG('#draw').size(WIDTH, HEIGHT);
      const view = draw.group().scale(40).translate(50, 50);

      const s = initialize(view);

      console.log("initialized");
    });
});
