use <box.scad>
use <back.scad>
use <button.scad>
use <epaper.scad>
use <speaker.scad>
use <blue-pill.scad>
include <params.scad>
//include <printing.scad>

box();

epaper_placement() epaper();

speaker_placement() speaker();

button_placement() button(thickness);

translate([bluepill_x, bluepill_y, thickness + 53/2 + 2])
rotate([-90, 90, 0])
blue_pill();

translate([0,0,box_pilone_height]) back();

// coin baterry
//translate([bluepill_x-23.5,-box_height/2+2,10+2]) rotate([-90,0,0]) color([0.7, 0.7, 0.7]) cylinder(d=20, h=3.2);

/*
translate([0, box_height/2, box_depth - 28/2])
rotate([90,0,0])
color([0.1,0.1,0.1])
cube([100,28,2], center=true);
*/
