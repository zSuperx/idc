fn main(argc: i32, argv: **u8) -> u64 {
  if argc == 1 {
    return 0;
  } else if argc == 2 {
    return 4 + 5;
  } else if argc == 3 {
    return @u64(**argv);
  } else {
    return 69;
  }
}
