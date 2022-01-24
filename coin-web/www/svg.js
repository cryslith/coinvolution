export function get_location(svg, event) {
  return svg.point(event.pageX, event.pageY);
}
