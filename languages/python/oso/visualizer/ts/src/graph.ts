/*
 * # TODO
 *
 * - multi-line nodes should be vertically centered on the circle
 *    - after that's done, node heights can be ~halved
 * - maybe make it so graph can't be fully out of viewport?
 * - viewport should stretch the width of the browser
 * - maybe memoize some computations?
 **/

import * as d3 from 'd3';
import { flextree } from 'd3-flextree';

const OSO_BLUE_DARKEST = '#011f4b';
const OSO_BLUE_LIGHTEST = '#b3cde0';

interface CodeContext {
  lineno: number;
  column: number;
  filename: string;
}

interface Node {
  id: number;
  name: string;
  codeContext: CodeContext | null;
  root: boolean;
}

interface Link {
  source: string;
  target: string;
}

async function renderGraph(res: string) {
  const parentDiv = document.getElementById('d3-graph');
  if (!parentDiv) {
    console.error('Parent div not loaded!');
    return;
  }

  if (res.status !== 200) {
    const div = document.createElement('div');
    div.innerText = 'Not Allowed';
    parentDiv.appendChild(div);
    return;
  }

  const json = await res.json();
  const nodes: Node[] = json.nodes.map(([id, name, codeContext, root]) => ({
    id,
    name: name.replace(/[<>]/g, match => (match === '>' ? '&gt;' : '&lt;')),
    codeContext: JSON.parse(codeContext),
    root,
  }));
  const links: Link[] = json.edges.map(([source, target]) => ({
    source,
    target,
  }));
  const chart = buildChart({ nodes, links });
  parentDiv.appendChild(chart);
}

async function audit(id: int) {
  const res = await fetch(`../event_json/${id}`);
  renderGraph(res);
}

async function graph() {
  // @TODO(Sam): parse the location based on just the last
  // segments. Necessary since flask viz can be on custom path
  const segments = window.location.pathname.split('/');
  const last = segments.pop();
  try {
    if (segments.pop() === 'events') {
      audit(parseInt(last!));
    } else {
      console.log('WAT');
    }
  } catch (e) {
    console.error(e);
  }
}

interface Hierarchy {
  name: string;
  children: Hierarchy[];
}

function toHierarchy(links: Link[], nodes: Node[]): Hierarchy {
  const root = nodes.find(n => n.root);
  const hierarchy: Hierarchy = { ...root };

  function walk(h: Hierarchy) {
    const children = links
      .filter(l => l.source === h.id)
      .map(l => nodes.find(n => l.target === n.id))
      .map(n => walk({ ...n }));
    return {
      ...h,
      children,
    };
  }

  return walk(hierarchy);
}

function tspan_str(s: string) {
  if (s.indexOf("\n") === -1) {
    return s;
  } else {
    const spans = s.split("\n");
    const head = `<tspan dy="1em">${spans[0]}</tspan>`;
    const rest = spans
      .slice(1)
      .map((s) => `<tspan x="2em" dy="1em">${s}</tspan>`);
    return head + rest.join("");
  }
}

function display(node, truncate) {
  const { name } = node.data;
  if (!node._children || truncate === false) {
    return tspan_str(name);
  }

  const result = /^(\w+)\((.*)\)$/.exec(name);
  if (!result || result.length !== 3) {
    return tspan_str(name);
  }
  const [original, head, _body] = result;

  if (truncate === true || node.children) {
    return tspan_str(`${head}(...)`);
  }

  return tspan_str(original);
}

function nodeWidth(node): number {
  return display(node).length + 1;
}

const buildChart = ({ links, nodes }) => {
  const width = 1200;

  const margin = { top: 12, right: 0, bottom: 12, left: 180 };
  const dx = 100;

  const tree = flextree().nodeSize(d => {
    const height = 50;
    const minWidth = 10;
    let cols: number;
    // If the current node (R) is the root node, its width should be...
    if (!d.parent) {
      // the width of its widest child (Y):
      // R () ----- Y () -----
      //      ^-----^
      cols = Math.max(...d.children.map(c => nodeWidth(c)));
      // Else, if the current node (X) doesn't have any children, its width should be...
    } else if (!d.children) {
      // the width of its widest childless elder (Y) (since a childless elder's content will be on the right side of its node)...
      const prevGenRHS = Math.max(
        0,
        ...d.parent.parent.children
          .filter(elder => !elder.children)
          .map(elder => nodeWidth(elder))
      );
      // plus the width of the widest member of its own generation who has children (Z) (since their content will be on the left side of their nodes)...
      //           ------ Z ()
      //         /
      // ----- () Y ------- () X
      //          ^-------^
      const sameGenLHS = Math.max(
        0,
        ...d.parent.parent.children
          .filter(elder => elder.children)
          .flatMap(elder => elder.children.filter(child => child.children))
          .map(child => nodeWidth(child))
      );
      // or `minWidth`, whichever is greater:
      cols = Math.max(minWidth, prevGenRHS + sameGenLHS);
      // Else, if the current node (X) has children, its width should be...
    } else {
      // the width of the widest member of the next generation who has children (Y)...
      const nextGenLHS = Math.max(
        0,
        ...d.parent.children
          .filter(current => current.children)
          .flatMap(current =>
            current.children
              .filter(next => next.children)
              .map(next => nodeWidth(next))
          )
      );
      // plus the width of the widest childless member of its own generation (Z)...
      // --- X () ------- Y ()
      // ----- () Z
      //          ^-------^
      const sameGenRHS = Math.max(
        0,
        ...d.parent.children
          .filter(current => !current.children)
          .map(current => nodeWidth(current))
      );
      // or `minWidth`, whichever is greater:
      cols = Math.max(minWidth, nextGenLHS + sameGenRHS);
    }
    const width = Math.max(cols, 10) * 12;
    return [height, width];
  });

  const diagonal = d3
    .linkHorizontal()
    .x(d => d.y)
    .y(d => d.x);

  const hierarchy = toHierarchy(links, nodes);
  const root = tree.hierarchy(hierarchy);

  root.x0 = 0;
  root.y0 = 0;
  root.descendants().forEach((d, i) => {
    d.id = i;
    d._children = d.children;
    // This determines whether the tree starts partially (or fully) folded
    if (d.depth > 1) d.children = null;
  });

  const svg = d3
    .create('svg')
    .style('font', '14px monospace')
    .style('border-top', `2px ${OSO_BLUE_DARKEST} solid`)
    .style('user-select', 'none');

  const g = svg.append('g');

  const zoom = d3
    .zoom()
    .scaleExtent([0.5, 4])
    .on('zoom', zoomed);

  svg.call(zoom);

  function reset() {
    svg
      .transition()
      .duration(500)
      .call(
        zoom.transform,
        d3.zoomIdentity,
        d3.zoomTransform(svg.node()).invert([width / 2, dx / 2])
      );
  }

  function zoomed() {
    const { transform } = d3.event;
    g.attr('transform', transform);
  }

  // Reset zoom with ESC
  window.onkeydown = ({ keyCode }) => (keyCode === 27 && reset()) || null;

  const gLink = g
    .append('g')
    .attr('fill', 'none')
    .attr('stroke', OSO_BLUE_LIGHTEST)
    .attr('stroke-opacity', 0.7)
    .attr('stroke-width', 1.5);

  const gNode = g
    .append('g')
    .attr('cursor', 'pointer')
    .attr('pointer-events', 'all');

  function update(source) {
    const duration = d3.event && d3.event.altKey ? 2500 : 250;
    const nodes = root.descendants().reverse();
    const links = root.links();

    // Compute the new tree layout.
    tree(root);

    let left = Infinity;
    let right = -Infinity;
    root.eachBefore(node => {
      const height = 50;
      const x = node.x + height;
      if (x < left) left = x;
      if (x > right) right = x;
    });

    let top = Infinity;
    let bottom = -Infinity;
    root.eachBefore(node => {
      const rhs = node.children ? 0 : nodeWidth(node) * 12;
      const y = node.y + rhs;
      if (y < top) top = y;
      if (y > bottom) bottom = y;
    });

    const height = right - left + margin.top + margin.bottom;
    const width = bottom - top + margin.left + margin.right;

    const transition = svg
      .transition()
      .duration(duration)
      .attr('viewBox', [-margin.left, -height / 2, width, height])
      .tween(
        'resize',
        window.ResizeObserver ? null : () => () => svg.dispatch('toggle')
      );

    // Update the nodes…
    const node = gNode.selectAll('g').data(nodes, d => d.id);

    // Enter any new nodes at the parent's previous position.
    const nodeEnter = node
      .enter()
      .append('g')
      .attr('transform', _d => `translate(${source.y0},${source.x0})`)
      .attr('fill-opacity', 0)
      .attr('stroke-opacity', 0)
      .on('click', d => {
        d.children = d.children ? null : d._children;
        update(d);
        updateText();
        updateCircle();
      });

    nodeEnter.append('title').text(d => d.data.name);

    nodeEnter
      .append('circle')
      .attr('r', 5)
      .attr('stroke', d => (d._children ? OSO_BLUE_DARKEST : '#999'))
      .attr('stroke-width', 2)
      .attr('fill', d => {
        if (d._children === null) {
          return '#999';
        } else if (d.children) {
          return '#fff';
        } else {
          return OSO_BLUE_DARKEST;
        }
      });

    const nodeText = nodeEnter.append('a');

    nodeText
      .append('text')
      .attr('class', 'unfolded')
      .attr('dy', '0.31em')
      .attr('x', d => (d.children ? '-1em' : '1em'))
      .attr('text-anchor', d => (d.children ? 'end' : 'start'))
      .attr('fill', OSO_BLUE_DARKEST)
      .html(d => display(d, false))
      .attr('visibility', d => (d.children ? 'hidden' : 'visible'))
      .clone(true)
      .lower()
      .attr('stroke-linejoin', 'round')
      .attr('stroke-width', 3)
      .attr('stroke', 'white');

    nodeText
      .append('text')
      .attr('class', 'folded')
      .attr('dy', '0.31em')
      .attr('x', d => (d.children ? '-1em' : '1em'))
      .attr('text-anchor', d => (d.children ? 'end' : 'start'))
      .attr('fill', OSO_BLUE_DARKEST)
      .html(d => display(d, true))
      .attr('visibility', d => (d.children ? 'visible' : 'hidden'))
      .clone(true)
      .lower()
      .attr('stroke-linejoin', 'round')
      .attr('stroke-width', 3)
      .attr('stroke', 'white');

    function updateText() {
      d3.selectAll('.folded')
        .transition(transition)
        .attr('x', d => (d.children ? '-1em' : '1em'))
        .attr('text-anchor', d => (d.children ? 'end' : 'start'))
        .attr('visibility', d => (d.children ? 'visible' : 'hidden'));
      d3.selectAll('.unfolded')
        .transition(transition)
        .attr('x', d => (d.children ? '-1em' : '1em'))
        .attr('text-anchor', d => (d.children ? 'end' : 'start'))
        .attr('visibility', d => (d.children ? 'hidden' : 'visible'));
    }

    function updateCircle() {
      d3.selectAll('circle')
        .transition(transition)
        .attr('fill', d => {
          if (d._children === null) {
            return '#999';
          } else if (d.children) {
            return '#fff';
          } else {
            return OSO_BLUE_DARKEST;
          }
        });
    }

    // Transition nodes to their new position.
    node
      .merge(nodeEnter)
      .transition(transition)
      .attr('transform', d => `translate(${d.y},${d.x})`)
      .attr('fill-opacity', 1)
      .attr('stroke-opacity', 1);

    // Transition exiting nodes to the parent's new position.
    node
      .exit()
      .transition(transition)
      .remove()
      .attr('transform', _d => `translate(${source.y},${source.x})`)
      .attr('fill-opacity', 0)
      .attr('stroke-opacity', 0);

    // Update the links…
    const link = gLink.selectAll('path').data(links, d => d.target.id);

    // Enter any new links at the parent's previous position.
    const linkEnter = link
      .enter()
      .append('path')
      .attr('d', _d => {
        const o = { x: source.x0, y: source.y0 };
        return diagonal({ source: o, target: o });
      });

    // Transition links to their new position.
    link
      .merge(linkEnter)
      .transition(transition)
      .attr('d', diagonal);

    // Transition exiting nodes to the parent's new position.
    link
      .exit()
      .transition(transition)
      .remove()
      .attr('d', _d => {
        const o = { x: source.x, y: source.y };
        return diagonal({ source: o, target: o });
      });

    // Stash the old positions for transition.
    root.eachBefore(d => {
      d.x0 = d.x;
      d.y0 = d.y;
    });
  }

  update(root);

  return svg.node();
};

graph();
