#import "label.typ"

#label.label(
  width: int(sys.inputs.at("width", default: "696")) * 1pt,
  height: int(sys.inputs.at("height", default: "300")) * 1pt,
  name: sys.inputs.at("media", default: "C62"),
  color_support: sys.inputs.at("color_support", default: "true") == "true",
)
