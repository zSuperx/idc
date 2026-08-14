fn main() -> i32 {
  let x = 10;
  let i = 0;
  let a = 0;
  let b = 1;
  while i < x {
    i = i + 1;
    let tmp = a + b;
    a = b;
    b = tmp;
  }
  return b;
}
