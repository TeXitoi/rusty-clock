use <utils.scad>
use <screws.scad>

module speaker_holes() {
  for (r=[45:90:360])
    rotate([0,0,r])
      translate([30,0,0])
      children();
}

module speaker(support_thickness=2) {
  difference() {
    union() {
      translate([0, 0, -0.3])
        linear_extrude(0.3)
        rounded_square([53, 53], r=8, center=true);
      translate([0, 0, -17]) cylinder(d1=35, d2=52, h=17);
      translate([0, 0, -28]) cylinder(d=35, h=28);
      color([0.2, 0.2, 0.2]) translate([0, 0, -18 - 8]) cylinder(d=40, h=8);
    }
    translate([0,0,28]) color([0.7,0.7,0.7]) sphere(r=35);
    cylinder(d=47, h=4, center=true);

    // holes
    for (r=[45:90:360]) {
      rotate([0,0,r]) translate([30,0,0]) hull() {
        for (t=[-0.75,0.75])
          translate([t, 0, 0])
            cylinder(d=4.5, h=1, center=true);
      }
    }
  }

  // tore
  color([0.2, 0.2, 0.2])
    translate([0,0,-2])
    rotate_extrude()
    translate([20,0,0])
    rotate([0,0,90])
    circle(d=5.5);

  color([0.2, 0.2, 0.2]) difference() {
    cylinder(d=50, h=3, center=true);
    cylinder(d=47, h=4, center=true);
  }

  speaker_holes() {
    translate([0, 0, support_thickness]) m3_screw();
    translate([0, 0, -0.3]) m3_washer();
    translate([0, 0, -0.3-0.5]) m3_bolt();
  }
}

module speaker_pocket() {
  translate([0, 0, -60/2]) cube([54, 54, 60], center=true);
  cylinder(d=51, h=20, center=true);
  speaker_holes() cylinder(d=3.5, h=20, center=true);
}

speaker();
//speaker_pocket();
