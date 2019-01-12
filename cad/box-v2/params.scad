box_width=120;
box_height=70;
box_depth=40;
box_rounding=6;
thickness=2;
button_spacing=(box_width - 2*box_rounding) / 4;
button_coords=[ for (i=[0:3]) [(-1.5 + i)*button_spacing, box_height/2, box_depth/2] ];
