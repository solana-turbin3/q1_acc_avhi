use super::helper::*;

#[test]
fn test_init_extra_acc_meta() {
    let (mut svm, admin) = setup();
    let (_vault_pda, _mint_kp, mint_pk) = do_initialize(&mut svm, &admin);

    let extra_acc_meta_list = do_init_extra_acc_meta(&mut svm, &admin, &mint_pk);

    let account = svm.get_account(&pubkey_to_addr(&extra_acc_meta_list));
    assert!(account.is_some(), "Extra account meta list should exist");
}
