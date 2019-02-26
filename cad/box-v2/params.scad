// bluepill including tolerance
bluepill_width=53;
bluepill_height=23;

thickness=2;

backpanel_pilone_size=7;
backpanel_insertion_size=1;

box_width=155;
box_height=54+2*(thickness);
box_depth=bluepill_width + 2 + 2*thickness + backpanel_insertion_size;
box_rounding=6;
backpanel_hole_coords=[
  for (i=[-1,1])
    for (j=[-1,1])
      [i * (box_width/2-thickness-backpanel_pilone_size+2.5),
       j * (box_height/2-thickness-backpanel_pilone_size+2.5)]
];

module button_placement() {
  button_spacing=25;
  button_x=box_width/2-9-2*button_spacing;
  for (coord = [ for (i=[-1.5:1.5]) [i*button_spacing+button_x, box_height/2, 20] ])
    translate(coord)
      rotate([-90, 0, 0])
      children();
}

module epaper_placement() {
  x_placement=box_width/2-thickness-90.5/2-5.5/2-2;
  y_placement=box_height/2-thickness-39/2-2;
  translate([x_placement, y_placement, 1]) rotate([0,180,0]) children();
}

module speaker_placement() {
  translate([-box_width/2+54/2+thickness, 0, thickness]) rotate([0,180,0]) children();
}
