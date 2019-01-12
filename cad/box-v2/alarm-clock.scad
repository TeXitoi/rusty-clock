use <box.scad>
use <button.scad>
include <params.scad>

$fs=0.5;
$fa=1;

box();

for (coord = button_coords)
  translate(coord)
    rotate([-90, 0, 0])
    button();
