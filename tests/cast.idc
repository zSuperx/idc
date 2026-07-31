fn main() {
  let x: u64 = 0;
  let a = @(*u8)&x;
  a[0] = 5;
}
