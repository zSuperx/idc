fn main(argc: i32) -> u8 {
  if argc == 1 {
    return 69;
  } else {
    return @u8(sizeof(69));
  }
}
