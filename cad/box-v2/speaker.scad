module speaker() {
  difference() {
    union() {
      translate([0, 0, 0.3/2]) cube([53, 53, 0.3], center=true);
      cylinder(d1=52, d2=35, h=17);
      cylinder(d=35, h=28);
      color([0.2, 0.2, 0.2]) translate([0, 0, 18]) cylinder(d=40, h=8);
    }
    translate([0,0,-28]) color([0.7,0.7,0.7]) sphere(r=35);
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
    translate([0,0,2])
    rotate_extrude()
    translate([20,0,0])
    rotate([0,0,90])
    circle(d=5.5);

  color([0.2, 0.2, 0.2]) translate([0,0,0]) difference() {
    cylinder(d=50, h=3, center=true);
    cylinder(d=47, h=4, center=true);
  }
}

speaker();
