

import init, { Square, Node, Link, Bundle, DiagramOpt, ElementOpt, Diagram, Point, LabelPosition, Animation, LinkSet } from '../../../pkg/diagram_r';
async function run() {


  let res = await init();

  const opt = new DiagramOpt();
  opt.link_scale = 1.0;
  const d = new Diagram(opt);
  /* 800x600
    width: 60px
    height: 60px,
   
   
  */
  const north = new Node(new Square(365, 10, 60, 60), "North", 0, new Uint32Array());
  const south = new Node(new Square(365, 530, 60, 60), "South", 1, new Uint32Array());
  const west = new Node(new Square(10, 265, 60, 60), "West", 2, new Uint32Array());
  const east = new Node(new Square(730, 265, 60, 60), "East", 3, new Uint32Array());

  d.set_element_options([
    new ElementOpt("images/router_up.svg", "lightgreen", LabelPosition.Top),
    new ElementOpt("images/router_down.svg", "pink", LabelPosition.Center),
    new ElementOpt("images/router_unknown.svg", "lightblue", LabelPosition.Center),
    new ElementOpt("images/firewall_down.svg", "pink", LabelPosition.Bottom),
    //new ElementOpt("images/bad_file.svg", "pink", LabelPosition.Bottom),
  ],
  )


  let link_north_south = new Link(0, "North to South", Animation.Both);
  let link_south_north = new Link(0, "South to North", Animation.ToDst);
  let link_south_north2 = new Link(0, "South to North", Animation.ToDst);
  let nts = new LinkSet([
    link_north_south,
    link_south_north,
    link_south_north2,
  ], [], 0, 1);
  let link_east_west = new Link(1, "East to West", Animation.Both);
  let etw = new LinkSet([link_east_west], [], 2, 3)

  d.set_data([], [north, south, east, west], [nts, etw]);

  const el = document.getElementById("app") as HTMLCanvasElement;
  d.mount(el);
  d.render();

}

run()