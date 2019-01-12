module button(thickness=1) {
  color([0.7, 0.7, 0.7]) {
     // cap
    cylinder(d=17.7, h=0.4);
    translate([0, 0, 0.4]) cylinder(d1=17.7, d2=13.2, h=2-0.4);

    cylinder(d=11.4, h=4); // button
    translate([0, 0, -12]) cylinder(d=15.7, h=12); // thread

    // bolt
    translate([0, 0, -thickness - 2.6/2]) {
      intersection() {
        cylinder(d=21.2, h=2.6, $fn=6, center=true);
        resize([22, 22, 4.5]) sphere(d=24);
      }
    }

    // connectors
    for (y = [-3, 3]) {
      translate([0, y, -20 / 2]) cube([4, 4, 20], center=true);
      translate([0, y, -21.6])cylinder(d=4, h=21.6);
    }
  }

  // blue plastic
  color([0.2, 0.2, 1])
    translate([0, 0, -12-2])
      cylinder(d=11.7, h=2);
}

button();
