

import init, { Square, Node, Link, Bundle, DiagramOpt, ElementOpt, Diagram, Point, } from '../../../pkg/diagram_r';
async function run() {


  let res = await init();

  const opt = new DiagramOpt();
  const d = new Diagram(opt);
  /* 800x600
    width: 30px
    height: 30px,
   
   
  */
  const north = new Node(new Square(385, 10, 30, 30), "North", 0, new Uint32Array());
  const south = new Node(new Square(385, 560, 30, 30), "South", 0, new Uint32Array());
  const west = new Node(new Square(10, 285, 30, 30), "West", 0, new Uint32Array());
  const east = new Node(new Square(760, 285, 30, 30), "East", 0, new Uint32Array());


  d.set_data([], [north, south, east, west], []);

  d.mount("app");
  d.render();

}

run()