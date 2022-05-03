export function get_location(svgSelector, event) {
  const svg = document.querySelector(svgSelector);
  const pt = svg.createSVGPoint();
  pt.x = event.clientX;
  pt.y = event.clientY;
  return pt.matrixTransform(svg.getScreenCTM().inverse());
}
