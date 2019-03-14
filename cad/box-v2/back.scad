use <utils.scad>
include <params.scad>

function hex_dist(d) = sqrt(pow(d, 2)-pow(d/2, 2));

module back() {
    width=box_width-2*thickness-1;
    height=box_height-2*thickness-1;
    rounding=box_rounding-thickness-0.5;
    hole_d=3;
    usb_hole_coord=[bluepill_x, bluepill_y + (2.6+1.6)/2, 0];

    color(box_color) {
        difference() {
            intersection() {
                union() {
                    difference() {
                        linear_extrude(thickness)
                            rounded_square([width, height], r=rounding, center=true);
                        for (x=[-6:5])
                            for (y=[-4:4]) {
                                size=10.1;
                                translate([x*(size*1.5), y*hex_dist(size), 0]) {
                                    cylinder(d=size-thickness, h=3*thickness, center=true, $fn=6);
                                    translate([(size*1.5)/2, hex_dist(size)/2, 0])
                                        cylinder(d=size-thickness, h=3*thickness, center=true, $fn=6);
                                }
                            }
                    }
                
                    difference() {
                        linear_extrude(thickness)
                            rounded_square([width, height], r=rounding, center=true);
                        translate([0,0,-thickness])
                            linear_extrude(thickness*3)
                            rounded_square([width-4*thickness, height-4*thickness], r=rounding, center=true);
                    }
                    for (coord=backpanel_hole_coords) {
                        translate(coord) cylinder(d=hole_d+1+4*thickness, h=3*thickness, center=true);
                    }
                    translate(usb_hole_coord) {
                        cube([4, 8+4*thickness, thickness*3], center=true);
                        for (x=[-2, 2])
                            translate([x, 0, 0])
                                cylinder(d=8+4*thickness, h=thickness*3, center=true);
                    }
                }
                linear_extrude(thickness)
                    rounded_square([width, height], r=rounding, center=true);                
            }

            for (coord=backpanel_hole_coords) {
                translate(coord) {
                    cylinder(d=hole_d, h=3*thickness, center=true);
                    cylinder(r1=hole_d/2-1, r2=hole_d/2+thickness+1, h=thickness+1);
                }
            }

            translate(usb_hole_coord) {
                cube([4, 8, thickness*3], center=true);
                for (x=[-2, 2]) translate([x, 0, 0]) cylinder(d=8, h=thickness*3, center=true);
            }
        }
    }
}
