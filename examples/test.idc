fn main(argc: i32, argv: **u8) -> u8 {
    return argv[0][@(u64)argc - 1];
}
