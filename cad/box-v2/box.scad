use <utils.scad>
use <epaper.scad>
use <speaker.scad>
include <params.scad>

module box() {
  color([40/255, 177/255, 214/255]) {
    difference() {
      union() {
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
        }

        // backpanel pilones
        pilone_height=box_depth-thickness-backpanel_insertion_size;
        for (i=[-1, 1])
          for (j=[-1, 1]) {
            translate([i * (box_width/2-thickness-backpanel_pilone_size/2),
                       j * (box_height/2-thickness-backpanel_pilone_size/2),
                       pilone_height/2])
              cube([backpanel_pilone_size, backpanel_pilone_size, pilone_height], center=true);
          }

        // bluepill support
        for (i=[0, bluepill_height - thickness]) {
          translate([box_width/2-2*thickness-backpanel_pilone_size - i,
                     -box_height/2+backpanel_pilone_size,
                     0])
            cube([thickness, thickness, pilone_height]);
          translate([box_width/2-2*thickness-backpanel_pilone_size - i,
                     -box_height/2+thickness,
                     0])
            cube([thickness, backpanel_pilone_size-2*thickness, pilone_height]);
        }
        translate([box_width/2-2*thickness-backpanel_pilone_size - bluepill_height,
                   -box_height/2+thickness,
                   0])
          cube([thickness, backpanel_pilone_size, pilone_height]);
        translate([box_width/2-bluepill_height-thickness-backpanel_pilone_size,
                   -box_height/2+thickness,
                   0])
          cube([bluepill_height, backpanel_pilone_size, thickness + 2]);
      }

      // button holes
      button_placement()
        cylinder(d=16.2, h=3*thickness, center=true);

      // epaper pocket
      epaper_placement()
        epaper_pocket();

      // backpanel screw holes
      for (coord=backpanel_hole_coords)
        translate([coord.x, coord.y, box_depth - 10])
          cylinder(d=1.8, h=10);

      // debugger pocket
      translate([box_width/2-thickness-backpanel_pilone_size-bluepill_height/2,
                 -box_height/2-thickness+backpanel_pilone_size+1.6+2.54/2,
                 (thickness+2+2)/2-1])
        cube([11, 3, thickness + 2 + 2], center=true);

      // speaker_pocket
      speaker_placement() speaker_pocket();
    }
  }
}

box();
