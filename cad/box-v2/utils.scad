module rounded_square(size=1, center=false, r=1) {
  offset(r)
    offset(-r)
      square(size, center=center);
}
