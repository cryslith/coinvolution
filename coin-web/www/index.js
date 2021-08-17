import init, {initialize_puzzle, count_darts, make_face_clickers, set_panic_hook} from "../pkg/coin_web.js";

const WIDTH = 1000, HEIGHT = 500;

SVG.on(document, 'DOMContentLoaded', () => {
  init()
    .then(() => {
      set_panic_hook();
      
      const draw = SVG('#draw').size(WIDTH, HEIGHT);
      const view = draw.group().scale(40).translate(50, 50);

      const p = initialize_puzzle(); 
      const state = {
        svg: view,
        puzzle: p
      };

      console.log(p);
      count_darts(p);
      make_face_clickers(state, p);
    });
});
