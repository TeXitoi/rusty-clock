use <utils.scad>
include <params.scad>

function hex_dist(d) = sqrt(pow(d, 2)-pow(d/2, 2));

module back() {
    tolerance=0.5;
    width=box_width-2*thickness-tolerance;
    height=box_height-2*thickness-tolerance;
    rounding=box_rounding-thickness-tolerance/2;
    hole_d=3;
    usb_hole_coord=[bluepill_x, bluepill_y + (2.6+1.6)/2, 0];
    pattern_thickness=1.2;
    border=1.5*pattern_thickness;
    pattern_size=10;

    color(box_color) {
        difference() {
            intersection() {
                union() {
                    difference() {
                        linear_extrude(pattern_thickness)
                            rounded_square([width, height], r=rounding, center=true);
                        for (x=[-6:5])
                            for (y=[-4:4]) {
                                translate([x*(pattern_size*1.5), y*hex_dist(pattern_size), 0]) {
                                    cylinder(d=pattern_size-pattern_thickness, h=3*pattern_thickness, center=true, $fn=6);
                                    translate([(pattern_size*1.5)/2, hex_dist(pattern_size)/2, 0])
                                        cylinder(d=pattern_size-pattern_thickness, h=3*pattern_thickness, center=true, $fn=6);
                                }
                            }
                    }
                
                    difference() {
                        linear_extrude(pattern_thickness)
                            rounded_square([width, height], r=rounding, center=true);
                        translate([0,0,-pattern_thickness])
                            linear_extrude(pattern_thickness*3)
                            rounded_square([width-2*border, height-2*border], r=rounding, center=true);
                    }
                    for (coord=backpanel_hole_coords) {
                        translate(coord) cylinder(d=hole_d+2+2*border, h=3*pattern_thickness, center=true);
                    }
                    translate(usb_hole_coord) {
                        cube([4, 8+2*border, pattern_thickness*3], center=true);
                        for (x=[-2, 2])
                            translate([x, 0, 0])
                                cylinder(d=8+2*border, h=pattern_thickness*3, center=true);
                    }
                }
                linear_extrude(pattern_thickness)
                    rounded_square([width, height], r=rounding, center=true);                
            }

            for (coord=backpanel_hole_coords) {
                translate(coord) {
                    cylinder(d=hole_d, h=3*pattern_thickness, center=true);
                    cylinder(r1=hole_d/2-1, r2=hole_d/2+pattern_thickness+1, h=pattern_thickness+1);
                }
            }

            translate(usb_hole_coord) {
                cube([4, 8, pattern_thickness*3], center=true);
                for (x=[-2, 2]) translate([x, 0, 0]) cylinder(d=8, h=pattern_thickness*3, center=true);
            }
        }
    }
}
