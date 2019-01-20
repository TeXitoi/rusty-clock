module m3_screw(h=6) {
  color([0.7,0.7,0.7]) {
    cylinder(d=5, h=1.3);
    translate([0, 0, -h])
      cylinder(d=2.7, h=h);
  }
}

module m3_bolt(h=2.5) {
  color([0.7,0.7,0.7]) translate([0, 0, -h]) cylinder(d=5, h=h, $fn=6);
}

module m3_washer() {
  color([0.7,0.7,0.7]) translate([0, 0, -0.5/2]) {
    difference() {
      cylinder(d=7, h=0.5, center=true);
      cylinder(d=3.2, h=1, center=true);
    }
  }
}
