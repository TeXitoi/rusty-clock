// bluepill including tolerance
bluepill_width=53;
bluepill_height=23;

thickness=2;

backpanel_pilone_size=7;
backpanel_insertion_size=1;

box_width=120;
box_height=70;
box_depth=bluepill_width + 2 + 2*thickness + backpanel_insertion_size;
box_rounding=6;
button_spacing=(box_width - 2*box_rounding - 2) / 4;
button_coords=[ for (i=[0:3]) [(-1.5 + i)*button_spacing, box_height/2, box_depth/2] ];
backpanel_hole_coords=[
  for (i=[-1,1])
    for (j=[-1,1])
      [i * (box_width/2-thickness-backpanel_pilone_size+2.5),
       j * (box_height/2-thickness-backpanel_pilone_size+2.5)]
];

module epaper_placement() {
  epaper_height=box_height/2-thickness-39/2;
  translate([0,epaper_height,1]) rotate([0,180,0]) children();
}
