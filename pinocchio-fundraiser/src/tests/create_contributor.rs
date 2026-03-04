use super::helpers::setup_create_contributor;

#[test]
fn test_create_contributor() {
    let s = setup_create_contributor(1_000_000_000, 7, 500_000_000);
    println!("{:<20} | {:>6} CUs", "create_contributor", s.create_contributor_cu);
}
