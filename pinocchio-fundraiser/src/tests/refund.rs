use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::helpers::{TOKEN_PROGRAM_ID, program_id, setup_contribute};

#[test]
fn test_refund() {
    let goal = 1_000_000_000u64;
    let contributed = 500_000_000u64;

    let mut s = setup_contribute(goal, 1, contributed);

    // warp clock past the 1-day deadline (time_started=0, deadline=86400)
    let mut clock = s.svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 86401;
    s.svm.set_sysvar::<Clock>(&clock);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.contributor.pubkey(), true),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new(s.vault.pubkey(), false),
            AccountMeta::new(s.contributor_ata, false),
            AccountMeta::new(s.contributor_state_pda, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![3u8], // discriminator 3 = refund
    };

    let msg = Message::new(&[ix], Some(&s.contributor.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&s.contributor], msg, blockhash))
        .unwrap();

    println!("{:<12} | {:>6} CUs", "refund", tx.compute_units_consumed);
}
