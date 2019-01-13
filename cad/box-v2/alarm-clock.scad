use <box.scad>
use <button.scad>
use <epaper.scad>
include <params.scad>
include <printing.scad>

box();

epaper_placement() epaper();

for (coord = button_coords)
  translate(coord)
    rotate([-90, 0, 0])
    button(thickness);
