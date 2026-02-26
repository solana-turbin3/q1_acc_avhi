use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::super::helpers::{program_id, setup_make_v2, TOKEN_PROGRAM_ID};

#[test]
fn test_cancel() {
    let mut s = setup_make_v2(100_000_000, 500_000_000);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.escrow_pda, false),
            AccountMeta::new(s.maker_ata_a, false),
            AccountMeta::new(s.escrow_ata, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![5u8],
    };

    let msg = Message::new(&[ix], Some(&s.maker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.maker], msg, blockhash))
        .unwrap();

    println!("{:<12} | {:>6} CUs", "cancel v2", tx.compute_units_consumed);
}
