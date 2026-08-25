fn main(x: i32) -> i32 {
  let y = &x;
  *y = 69;
  return x + *y;
}
