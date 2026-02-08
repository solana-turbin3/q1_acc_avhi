use anchor_lang::AccountDeserialize;
use solana_signer::Signer;

use super::helper::*;

#[test]
fn test_initialize() {
    let (mut svm, admin) = setup();

    let (vault_pda, _mint_kp, mint_pk) = do_initialize(&mut svm, &admin);

    let vault_account = svm.get_account(&pubkey_to_addr(&vault_pda)).unwrap();
    let vault_data =
        crate::state::Vault::try_deserialize(&mut vault_account.data.as_ref()).unwrap();

    let admin_pk = addr_to_pubkey(&admin.pubkey());
    assert_eq!(vault_data.admin, admin_pk);
    assert_eq!(vault_data.mint, mint_pk);
}
