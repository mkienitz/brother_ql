#let label(width: length, height: length, color_support: bool, name: text) = {
  set page(width: width, height: height, margin: 0pt)

  let scale_name(target_width: ratio, name_text: str) = context {
    let actual_width = measure(box(name_text)).width
    let fac = (target_width.pt() / actual_width.pt()) * 100%
    let col = if color_support {
      red
    } else {
      black
    }
    scale(x: fac, y: fac, reflow: true, text(fill: col, name_text))
  }
  let scale_dims(target_height: ratio, dim_text: str) = context {
    let actual_height = measure(box(dim_text)).height
    let fac = (target_height.pt() / actual_height.pt()) * 100%
    scale(x: fac, y: fac, reflow: true, dim_text)
  }

  let width_text = "W" + str(width.pt())
  let height_text = "H" + str(height.pt())
  let dim_font_size = calc.min(15% * width, 15% * height)
  let _font_size = calc.min(15% * width, 15% * height)

  set text(font: "DejaVu Sans Mono")

  box(
    inset: 6pt,
    width: width,
    height: height,
    stroke: 4pt,
  )[
    // Center: medium name
    #place(center + horizon, box(
      scale_name(target_width: 55% * width, name_text: name),
    ))
    //Left side (rotated height)
    #place(left + horizon, rotate(reflow: true, -90deg, box(
      scale_dims(target_height: dim_font_size, dim_text: height_text),
    )))
    // Bottom side (width)
    #place(bottom + center, box(scale_dims(
      target_height: dim_font_size,
      dim_text: width_text,
    )))
  ]
}
