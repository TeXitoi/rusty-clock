box_width=120;
box_height=70;
box_depth=40;
box_rounding=6;
thickness=2;
button_spacing=(box_width - 2*box_rounding) / 4;
button_coords=[ for (i=[0:3]) [(-1.5 + i)*button_spacing, box_height/2, box_depth/2] ];
backpanel_pilone_size=7;
backpanel_insertion_size=1;

module epaper_placement() {
  epaper_height=box_height/2-thickness-39/2;
  translate([0,epaper_height,1]) rotate([0,180,0]) children();
}
