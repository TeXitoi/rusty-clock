use <utils.scad>
use <epaper.scad>
include <params.scad>

module box() {
  color([40/255, 177/255, 214/255]) {
    // around
    difference() {
      linear_extrude(box_depth)
        rounded_square([box_width, box_height], r=box_rounding, center=true);

      // interior
      translate([0,0,thickness])
        linear_extrude(box_depth)
        rounded_square([box_width - 2*thickness, box_height - 2*thickness],
                       r=box_rounding - thickness,
                       center=true);

      // button holes
      for (coord=button_coords) {
        translate(coord)
          rotate([90, 0, 0])
          cylinder(d=16.2, h=3*thickness, center=true);
      }

      // epaper pocket
      epaper_placement()
        epaper_pocket();
    }
  }
}

box();
