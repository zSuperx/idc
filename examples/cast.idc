fn main() {
  let uint: i32 = 5;
  let x = @(*u64)uint;
  x[0] = 5;
}
