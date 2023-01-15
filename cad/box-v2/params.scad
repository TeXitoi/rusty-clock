thickness=0.6*3;

box_color=[40/255, 177/255, 214/255];

// bluepill including tolerance
bluepill_width=53.5;
bluepill_height=23;

backpanel_pilone_size=7;
backpanel_insertion_size=1;
box_width=155;
box_height=54+2*(thickness);
box_depth=bluepill_width + 2 + 2*thickness + backpanel_insertion_size;
box_rounding=6;
box_pilone_height=box_depth-thickness-backpanel_insertion_size;

bluepill_x=box_width/2-thickness-backpanel_pilone_size - 23/2;
bluepill_y=-box_height/2+backpanel_pilone_size - 0.5 * thickness;

backpanel_hole_coords=[
  for (i=[-1,1])
    for (j=[-1,1])
      [i * (box_width/2-thickness-backpanel_pilone_size+3),
       j * (box_height/2-thickness-backpanel_pilone_size+3)]
];

legend_depth=0.6;
legend_size=6;
legend_texts=["C", "<", ">", "OK"];
legend_font="Latin Modern Sans Quotation:style=8 Bold";
button_legend_offset=15;

//button_spacing=25;
//button_x=box_width/2-9-2*button_spacing;
button_spacing=19.05;
button_x=box_width/2-2*button_spacing-20;

button_coords=[ for (i=[1.5:-1:-1.5]) [i*button_spacing+button_x, box_height/2, box_depth/2] ];
module button_placement() {
  for (coord = button_coords)
    translate(coord)
      rotate([-90, 0, 0])
      children();
}

module epaper_placement() {
  x_placement=box_width/2-thickness-90.5/2-5.5/2-2;
  y_placement=box_height/2-thickness-39/2-2;
  translate([x_placement, y_placement, thickness-1]) rotate([0,180,0]) children();
}

module speaker_placement() {
  translate([-box_width/2+54/2+thickness, 0, thickness]) rotate([0,180,0]) children();
}
