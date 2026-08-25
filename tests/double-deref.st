fn test() -> i32 {
  let x = 5;
  let y = &x;
  let z = &y;
  *(*z) = 69;
  return x;
}
