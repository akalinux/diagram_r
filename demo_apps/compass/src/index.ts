

import init, { LinePoint, ArcType, Square, Node, Link, Bundle, DiagramOpt, ElementOpt, Diagram, Point, LabelPosition, Animation, LinkSet, GridOpt } from '../../../pkg/diagram_r';
async function run() {


  let res = await init();

  let opt = new DiagramOpt();
  opt.grid_opt = new GridOpt();
  opt.link_scale = 1;
  //opt.animate = false;
  const d = new Diagram(opt);

  const north = new Node(new Square(365, 20, 60, 60), "North", 0,);
  const south = new Node(new Square(365, 520, 60, 60), "South", 1);
  const west = new Node(new Square(10, 265, 60, 60), "West", 2,);
  //const west = new Node(new Square(0, 0, 60, 60), "West", 2,);
  const east = new Node(new Square(430, 265, 60, 60), "East", 3);
  const box = new Node(new Square(5, 5, 790, 590), "Container", 5, Uint32Array.of(0, 1, 2, 3));

  d.set_element_options([
    new ElementOpt("images/router_up.svg", "lightgreen", LabelPosition.Top),
    new ElementOpt("images/router_down.svg", "pink", LabelPosition.Top),
    new ElementOpt("images/router_unknown.svg", "lightblue", LabelPosition.Top),
    new ElementOpt("images/firewall_down.svg", "pink", LabelPosition.Bottom),
    new ElementOpt("images/bundle.svg", "lightblue", LabelPosition.Bottom),
    new ElementOpt("", "lightblue", LabelPosition.Center),
    new ElementOpt("", "pink", LabelPosition.Center),
  ],
  )


  const bundle = new Bundle(4, "First two", Uint32Array.from([0, 1]), 0.25)
  const bundle2 = new Bundle(4, "Outside Pairs", Uint32Array.from([0, 2]), 0.75)
  const etw = new LinkSet([
    new Link(0, "Both", Animation.Both), // 0
    new Link(0, "West to East", Animation.ToDst),  // 1
    new Link(0, "East to West", Animation.ToSrc),  // 2
    new Link(6, "Dead", Animation.None),  // 2
  ], [bundle, bundle2], 2, 3);
  const nts = new LinkSet(
    [
      new Link(0, "Both", Animation.Both),
      new Link(0, "South To North", Animation.ToSrc),
    ], [new Bundle(4, "Both", Uint32Array.from([0, 1]), 0.25)],
    0, 1,
    new LinePoint(new Point(700, 275), ArcType.Arc)
  );


  //               0      1      2     3
  d.set_data([box], [north, south, west, east], [
    etw,
    nts,
  ]);

  const el = document.getElementById("app") as HTMLCanvasElement;
  d.mount(el);
  d.render();

}

run()