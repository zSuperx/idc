fn test_store() {
  let x = @(**u64)0;
  **x = 69;
}

fn test_load() {
  let x = @(**u64)0;
  let y = **x;
}
