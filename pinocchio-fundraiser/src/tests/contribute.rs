use super::helpers::setup_contribute;

#[test]
fn test_contribute() {
    let s = setup_contribute(1_000_000_000, 7, 500_000_000);
    println!("{:<12} | {:>6} CUs", "contribute", s.contribute_cu);
}
