aaa_diameter=10.5;
aaa_height=44.5;

module aaa_battery() {
  color([0, 0.6, 0]) cylinder(d=aaa_diameter, h=aaa_height-0.8);
  color([0.7, 0.7, 0.7]) translate([0,0,0.5]) cylinder(d=3.8, h=aaa_height-0.5);
}

module aaa_holder(thickness=2, with_baterry=false) {
  tolerance=2;
  width=thickness*2+aaa_diameter;

  difference() {
    union() {
      translate([0,-width/2,0])
        cube([thickness*2+aaa_height+tolerance, width, thickness]);  
      for (i=[0, thickness+aaa_height+tolerance]) {
        translate([i,-width/2,thickness]) cube([thickness, width, 7]);
      }
      for (i=[-width/2:width-thickness:width/2]) {
        translate([0,i,thickness]) cube([aaa_height+tolerance+2*thickness, thickness, 7]);
      }
    }
    for (i=[thickness, thickness+tolerance+aaa_height-1])
      translate([i, -width, thickness]) cube([1, width*2, 5]);
  }
  if (with_baterry) {
    translate([thickness+tolerance/2, 0, thickness+aaa_diameter/2]) rotate([0,90,]) aaa_battery();
  }
}

aaa_holder(with_baterry=true);

