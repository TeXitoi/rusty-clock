use <utils.scad>
use <screws.scad>
include <params.scad>

width=89.5;
height=38;
hole_offset=2.5;
hole_x_offset = width / 2 - hole_offset;
hole_y_offset = height / 2 - hole_offset;
hole_offsets = [ for (x=[hole_x_offset,-hole_x_offset]) for (y=[hole_y_offset,-hole_y_offset]) [x, y, 0] ];
display_offset=-5.5/2;
display_size=[67, 29];

module epaper(support_thickness=2) {
  corner_radius=1.5;
  t=1.6;

  hole_diameter=3;

  translate([display_offset, 0, -1]) {
    difference() {
      // PCB
      color([0, 0, 0.6])
        translate([0, 0, -t])
        linear_extrude(t)
        rounded_square([width, height], center=true, r=corner_radius);

      // holes
      for (offset = hole_offsets)
        translate(offset)
          cylinder(d=3, h=3*t, center=true);
    }

    // glass
    color([0.9, 0.9, 0.9])
      translate([0, 0, 0.5])
      cube([79, 36.5, 1], center=true);

    // display ribbon
    color("gold")
      translate([-width/2 + 5.5/2 - 0.1, 0, -t/2])
      cube([5.6, 16, t + 2], center=true);

    // ribbon connector
    color([0.9, 0.9, 0.9])
      translate([width/2-7.5/2, 0, -t-5.5/2])
      cube([7.5, 20, 5.5], center=true);

    // ribbon
    color([0.4, 0, 0.4])
      translate([width/2+5, 0, -t-2])
      cube([10, 15, 1.2], center=true);

    for (offset = hole_offsets)
      translate(offset) {
        translate([0, 0, support_thickness]) m3_screw();
        color("gold") translate([0, 0, -t]) m3_bolt(h=5.7);
      }
  }

  // display area
  color([1, 1, 1]) cube([display_size.x, display_size.y, 0.01], center=true);
}

module epaper_pocket() {
  translate([display_offset, 0, -1]) {
    // holes
    for (offset = hole_offsets)
      translate(offset)
        cylinder(r=1.8, h=20, center=true);

    // pcb
    difference() {
      depth=45;
      os=5;
      translate([0,0,-depth]) linear_extrude(depth)
           rounded_square([width+os, height+os],
                          center=true,
                          r=box_rounding-thickness);
      for (pos=[[1,1,45], [-1,1,45+90], [-1,-1,45+180], [1,-1,45+270]]) {
        translate([pos.x*(width+os)/2, pos.y*(height+os)/2, -depth])
          rotate([0, 45, pos[2]])
            cube([10*sqrt(2),40,10*sqrt(2)], center=true);
      }
    }

    // glass
    cube([width-2*(hole_offset+2), height+1, 2], center=true);

    // display ribbon
    translate([-(width+1)/2, -10, 0])
      cube([20, 20, 1]);
  }

  // display_pocket
  chamfered_pocket([for (x=display_size) x+1], 3);
  cube([display_size.x+1, display_size.y+1, 1], center=true);
}

epaper();
#epaper_pocket();
