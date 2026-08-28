fn main() -> i32 {
  let x = 5;
  let y = 589;
  while (x < 69) {
    x = x + 1;
    y = x * 3 - y / 2;
  }
  return x;
}
