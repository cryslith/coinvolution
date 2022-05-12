export function client_to_svg(svgSelector, x, y) {
  const svg = document.querySelector(svgSelector);
  const pt = svg.createSVGPoint();
  pt.x = x;
  pt.y = y;
  return pt.matrixTransform(svg.getScreenCTM().inverse());
}
