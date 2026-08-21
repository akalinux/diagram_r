

import init, { Square, Node, Link, Bundle, DiagramOpt, ElementOpt, Diagram, Point, LabelPosition, Animation, LinkSet, GridOpt } from '../../../pkg/diagram_r';
async function run() {


  let res = await init();

  let opt = new DiagramOpt();
  opt.grid_opt = new GridOpt();
  const d = new Diagram(opt);
  /* 800x600
    width: 60px
    height: 60px,
   
   
  */
  const north = new Node(new Square(365, 20, 60, 60), "North", 0, new Uint32Array());
  const south = new Node(new Square(365, 520, 60, 60), "South", 1, new Uint32Array());
  const west = new Node(new Square(10, 265, 60, 60), "West", 2, new Uint32Array());
  const east = new Node(new Square(730, 265, 60, 60), "East", 3, new Uint32Array());

  d.set_element_options([
    new ElementOpt("images/router_up.svg", "lightgreen", LabelPosition.Top),
    new ElementOpt("images/router_down.svg", "pink", LabelPosition.Center),
    new ElementOpt("images/router_unknown.svg", "lightblue", LabelPosition.Center),
    new ElementOpt("images/firewall_down.svg", "pink", LabelPosition.Bottom),
    new ElementOpt("images/bundle.svg", "lightblue", LabelPosition.Bottom),
  ],
  )


  const bundle = new Bundle(4, "First two", Uint32Array.from([0, 1]), 0.25)
  const bundle2 = new Bundle(4, "Outside Pairs", Uint32Array.from([0, 2]), 0.75)
  const etw = new LinkSet([
    new Link(0, "Both", Animation.Both),
    new Link(0, "East to West", Animation.ToDst),
    new Link(0, "West to East", Animation.ToSrc),
  ], [bundle, bundle2], 2, 3);
  const nts = new LinkSet([
    new Link(1, "North To South", Animation.Both),
  ], [], 0, 1);

  d.set_data([], [north, south, east, west], [
    etw,
    nts,
  ]);

  const el = document.getElementById("app") as HTMLCanvasElement;
  d.mount(el);
  d.render();

}

run()