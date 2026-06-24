fn main(argc: i32, argv: **u8) -> u8 {
  if argc == 3 {
    return **argv;
  }
  return 0;
}
