use litesvm_token::CreateAssociatedTokenAccount;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::helpers::{TOKEN_PROGRAM_ID, program_id, setup_contribute};

#[test]
fn test_checker() {
    let amount = 1_000_000_000u64;
    let mut s = setup_contribute(amount, 7, amount);

    let maker_ata = CreateAssociatedTokenAccount::new(&mut s.svm, &s.maker, &s.mint)
        .owner(&s.maker.pubkey())
        .send()
        .unwrap();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new(s.vault, false),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![3u8], // discriminator 3 = checker
    };

    let msg = Message::new(&[ix], Some(&s.maker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.maker], msg, blockhash))
        .unwrap();

    println!("{:<20} | {:>6} CUs", "checker", tx.compute_units_consumed);
}
