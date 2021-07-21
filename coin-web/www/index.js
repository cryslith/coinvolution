import init, {initialize_graph, count_darts, set_panic_hook} from "../pkg/coin_web.js";
init()
  .then(() => {
    set_panic_hook();

    let g = initialize_graph();
    console.log(g);
    count_darts(g);
  });
