use super::helpers::setup_make;

#[test]
fn test_make() {
    let s = setup_make(100_000_000, 500_000_000);
    println!("Make OK | escrow_pda={} escrow_ata={}", s.escrow_pda, s.escrow_ata);
}
