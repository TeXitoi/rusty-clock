//$fs=1;

module epaper(support_thickness=2) {
  width=89.5;
  height=38;
  corner_radius=1.5;
  thickness=1.6;

  hole_offset=2.5;
  hole_diameter=3;
  hole_x_offset = width / 2 - hole_offset;
  hole_y_offset = height / 2 - hole_offset;
  hole_offsets = [ for (x=[hole_x_offset,-hole_x_offset]) for (y=[hole_y_offset,-hole_y_offset]) [x, y, 0] ];

  translate([-5.5/2, 0, -1]) {
    difference() {
      // PCB
      color([0, 0, 0.6])
        translate([0, 0, -thickness])
        linear_extrude(thickness)
        offset(corner_radius)
        offset(-corner_radius)
        square([width, height], center=true);

      // holes
      for (offset = hole_offsets)
        translate(offset)
          cylinder(d=3, h=3*thickness, center=true);
    }

    // glass
    color([0.9, 0.9, 0.9])
      translate([0, 0, 0.5])
      cube([79, 36.5, 1], center=true);

    // display ribbon
    color("gold")
      translate([-width/2 + 5.5/2 - 0.1, 0, -thickness/2])
      cube([5.6, 16, thickness + 2], center=true);

    // ribbon connector
    color([0.9, 0.9, 0.9])
      translate([width/2-7.5/2, 0, -thickness-5.5/2])
      cube([7.5, 20, 5.5], center=true);

    // ribbon
    color([0.4, 0, 0.4])
      translate([width/2+5, 0, -thickness-2])
      cube([10, 15, 1.2], center=true);

    for (offset = hole_offsets)
      translate(offset) {
        // screws
        color([0.7,0.7,0.7]) {
          translate([0, 0, support_thickness])
            cylinder(d=5, h=1.3);
          translate([0, 0, support_thickness - 6])
            cylinder(d=2.7, h=6);
        }

        // bolts
        color("gold") translate([0, 0, -5.7-thickness]) cylinder(d=5, h=5.7, $fn=6);
      }
  }

  // display area
  color([1, 1, 1]) cube([67, 29, 0.01], center=true);
}

epaper();
