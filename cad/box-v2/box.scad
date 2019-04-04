use <utils.scad>
use <epaper.scad>
use <speaker.scad>
use <battery.scad>
include <params.scad>

module box() {
  color(box_color) {
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
        for (i=[-1, 1])
          for (j=[-1, 1]) {
            translate([i * (box_width/2-thickness-backpanel_pilone_size/2),
                       j * (box_height/2-thickness-backpanel_pilone_size/2),
                       box_pilone_height/2])
              cube([backpanel_pilone_size, backpanel_pilone_size, box_pilone_height], center=true);
          }

        // bluepill support
        for (i=[0, bluepill_height - thickness]) {
          translate([box_width/2-2*thickness-backpanel_pilone_size - i,
                     -box_height/2+backpanel_pilone_size,
                     0])
            cube([thickness, thickness, box_pilone_height]);
          translate([box_width/2-2*thickness-backpanel_pilone_size - i,
                     -box_height/2+thickness,
                     0])
            cube([thickness, backpanel_pilone_size-2*thickness, box_pilone_height]);
        }
        bp_external_support_x=box_width/2-2*thickness-backpanel_pilone_size-bluepill_height;
        translate([bp_external_support_x,
                   -box_height/2+thickness,
                   0])
          cube([thickness, backpanel_pilone_size, box_pilone_height]);
        translate([box_width/2-bluepill_height-thickness-backpanel_pilone_size,
                   -box_height/2+thickness,
                   0])
          cube([bluepill_height, backpanel_pilone_size, thickness + 2]);

        // aaa-baterry holder
        for (i=[-0.5:-1:-2.5]) {
          translate([i*(10.5+thickness)+bp_external_support_x+0.5*thickness, -box_height/2, 0])
            rotate([-90, -90, 0])
            aaa_holder();
        }
        // coin-baterry holder
        translate([bp_external_support_x-3*10.5-3*thickness-20.5,
                   -box_height/2+backpanel_pilone_size,
                   thickness]){
          difference() {
            cube([20.5, thickness, 15]);
            translate([20/2, -thickness, 13]) cube([1, 3*thickness, 1]);
            translate([(20.5-14.5)/2, -thickness, 0]) cube([14.5, 3*thickness, 7]);
          }
        }
        translate([bp_external_support_x-3*10.5-4*thickness-20.5,
                   -box_height/2+thickness,
                   thickness]){
          cube([thickness, backpanel_pilone_size, 15]);
        }
      }// end of first element of difference

      // button holes
      button_placement()
        cylinder(d=16.2, h=3*thickness, center=true);

      // epaper pocket
      epaper_placement()
        epaper_pocket();

      // backpanel screw holes
      for (coord=backpanel_hole_coords)
        translate([coord.x, coord.y, box_depth - 19])
          cylinder(d=1.8, h=19);

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
