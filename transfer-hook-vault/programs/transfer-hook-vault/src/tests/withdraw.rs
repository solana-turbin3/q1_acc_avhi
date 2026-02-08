use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022::extension::StateWithExtensions;
use spl_token_2022::state::Account as TokenAccount;

use super::helper::*;

fn setup_for_withdraw() -> (
    litesvm::LiteSVM,
    Keypair,                      // admin
    Keypair,                      // user
    anchor_lang::prelude::Pubkey, // vault_pda
    anchor_lang::prelude::Pubkey, // mint_pk
    anchor_lang::prelude::Pubkey, // user_account_pda
    anchor_lang::prelude::Pubkey, // extra_acc_meta_list
    anchor_lang::prelude::Pubkey, // user_ata
    anchor_lang::prelude::Pubkey, // vault_ata
    u64,                          // deposited amount
) {
    let (mut svm, admin) = setup();
    let (vault_pda, _mint_kp, mint_pk) = do_initialize(&mut svm, &admin);

    let user = Keypair::new();
    let user_pk = addr_to_pubkey(&user.pubkey());
    svm.airdrop(&user.pubkey(), 5 * LAMPORTS_PER_SOL).unwrap();

    let user_account_pda = do_add_user(&mut svm, &admin, &vault_pda, &user_pk);
    let extra_acc_meta_list = do_init_extra_acc_meta(&mut svm, &admin, &mint_pk);

    let user_ata = do_create_ata(&mut svm, &user, &user_pk, &mint_pk);
    let vault_ata = do_create_ata(&mut svm, &admin, &vault_pda, &mint_pk);

    let mint_amount: u64 = 1_000_000_000_000;
    do_mint_tokens(&mut svm, &admin, &mint_pk, &user_ata, mint_amount);

    let deposit_amount: u64 = 500_000_000_000;

    let deposit_accounts = crate::accounts::Deposit {
        user: user_pk,
        vault: vault_pda,
        user_account: user_account_pda,
    }
    .to_account_metas(None);

    let deposit_ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: convert_account_metas(deposit_accounts),
        data: crate::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };

    let transfer_ix = build_transfer_checked_ix(
        &user_ata,
        &mint_pk,
        &vault_ata,
        &user_pk,
        deposit_amount,
        9,
        &extra_acc_meta_list,
        &user_account_pda,
    );

    send_ixs(&mut svm, &[deposit_ix, transfer_ix], &user, &[]);

    (
        svm,
        admin,
        user,
        vault_pda,
        mint_pk,
        user_account_pda,
        extra_acc_meta_list,
        user_ata,
        vault_ata,
        deposit_amount,
    )
}

#[test]
fn test_withdraw() {
    let (
        mut svm,
        _admin,
        user,
        vault_pda,
        mint_pk,
        user_account_pda,
        extra_acc_meta_list,
        user_ata,
        vault_ata,
        deposit_amount,
    ) = setup_for_withdraw();

    let user_pk = addr_to_pubkey(&user.pubkey());
    let withdraw_amount: u64 = 200_000_000_000;

    let withdraw_accounts = crate::accounts::Withdraw {
        user: user_pk,
        vault: vault_pda,
        user_account: user_account_pda,
        vault_token_account: vault_ata,
        token_program: token_2022_program_id(),
    }
    .to_account_metas(None);

    let withdraw_ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: convert_account_metas(withdraw_accounts),
        data: crate::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };

    let transfer_ix = build_transfer_checked_ix(
        &vault_ata,
        &mint_pk,
        &user_ata,
        &user_pk,
        withdraw_amount,
        9,
        &extra_acc_meta_list,
        &user_account_pda,
    );

    send_ixs(&mut svm, &[withdraw_ix, transfer_ix], &user, &[]);

    let vault_ata_account = svm.get_account(&pubkey_to_addr(&vault_ata)).unwrap();
    let vault_token_data =
        StateWithExtensions::<TokenAccount>::unpack(&vault_ata_account.data).unwrap();
    assert_eq!(
        vault_token_data.base.amount,
        deposit_amount - withdraw_amount
    );

    let user_ata_account = svm.get_account(&pubkey_to_addr(&user_ata)).unwrap();
    let user_token_data =
        StateWithExtensions::<TokenAccount>::unpack(&user_ata_account.data).unwrap();
    let mint_amount: u64 = 1_000_000_000_000;
    assert_eq!(
        user_token_data.base.amount,
        mint_amount - deposit_amount + withdraw_amount
    );

    let user_acc = svm.get_account(&pubkey_to_addr(&user_account_pda)).unwrap();
    let user_acc_data =
        crate::state::UserAccount::try_deserialize(&mut user_acc.data.as_ref()).unwrap();
    assert_eq!(user_acc_data.amount, deposit_amount - withdraw_amount);
}

#[test]
fn test_withdraw_insufficient_funds() {
    let (
        mut svm,
        _admin,
        user,
        vault_pda,
        _mint_pk,
        user_account_pda,
        _extra_acc_meta_list,
        _user_ata,
        vault_ata,
        deposit_amount,
    ) = setup_for_withdraw();

    let user_pk = addr_to_pubkey(&user.pubkey());
    let withdraw_amount = deposit_amount + 1;

    let withdraw_accounts = crate::accounts::Withdraw {
        user: user_pk,
        vault: vault_pda,
        user_account: user_account_pda,
        vault_token_account: vault_ata,
        token_program: token_2022_program_id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: convert_account_metas(withdraw_accounts),
        data: crate::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };

    let signers: Vec<&Keypair> = vec![&user];
    let message = Message::new(&[ix], Some(&user.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&signers, message, recent_blockhash);

    let res = svm.send_transaction(transaction);
    assert!(
        res.is_err(),
        "Withdraw should fail when amount exceeds deposited balance"
    );
}
