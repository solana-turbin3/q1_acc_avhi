use super::helpers::setup_make;

#[test]
fn test_make() {
    let s = setup_make(100_000_000, 500_000_000);
    println!("{:<12} | {:>6} CUs", "make v1", s.make_cu);
}
