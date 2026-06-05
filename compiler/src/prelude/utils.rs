pub fn align_n(x: i128, n: usize) -> i128 {
    (x + n as i128) & !n as i128
}

