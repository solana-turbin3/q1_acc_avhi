use super::helpers::setup_initialize;

#[test]
fn test_initialize() {
    let s = setup_initialize(1_000_000_000, 7);
    println!("{:<12} | {:>6} CUs", "initialize", s.init_cu);
}
