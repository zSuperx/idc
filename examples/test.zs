fn main(argc: i32) -> *i32 {
  let z = &argc;
  let y = &z;
  *y = z;
  return *y;
}
