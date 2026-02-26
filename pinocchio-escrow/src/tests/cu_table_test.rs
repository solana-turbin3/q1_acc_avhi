use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::helpers::{program_id, setup_make, setup_make_v2, MakeSetup, TOKEN_PROGRAM_ID};

fn do_take(s: &mut MakeSetup, disc: u8, amount_to_receive: u64) -> u64 {
    let taker = Keypair::new();
    s.svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_b)
        .owner(&s.maker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut s.svm, &s.maker, &s.mint_b, &taker_ata_b, amount_to_receive)
        .send()
        .unwrap();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(taker.pubkey(), true),
            AccountMeta::new(s.maker.pubkey(), false),
            AccountMeta::new(s.escrow_pda, false),
            AccountMeta::new(taker_ata_a, false),
            AccountMeta::new(taker_ata_b, false),
            AccountMeta::new(maker_ata_b, false),
            AccountMeta::new(s.escrow_ata, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![disc],
    };

    let msg = Message::new(&[ix], Some(&taker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = Transaction::new(&[&taker], msg, blockhash);
    s.svm.send_transaction(tx).unwrap().compute_units_consumed
}

fn do_cancel(s: &mut MakeSetup, disc: u8) -> u64 {
    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.escrow_pda, false),
            AccountMeta::new(s.maker_ata_a, false),
            AccountMeta::new(s.escrow_ata, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![disc],
    };

    let msg = Message::new(&[ix], Some(&s.maker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = Transaction::new(&[&s.maker], msg, blockhash);
    s.svm.send_transaction(tx).unwrap().compute_units_consumed
}

#[test]
fn test_cu_table() {
    let amount_to_receive = 100_000_000u64;
    let amount_to_give = 500_000_000u64;

    let mut s1 = setup_make(amount_to_receive, amount_to_give);
    let make_v1 = s1.make_cu;
    let take_v1 = do_take(&mut s1, 1, amount_to_receive);

    let mut s2 = setup_make_v2(amount_to_receive, amount_to_give);
    let make_v2 = s2.make_cu;
    let take_v2 = do_take(&mut s2, 4, amount_to_receive);

    let mut s3 = setup_make(amount_to_receive, amount_to_give);
    let cancel_v1 = do_cancel(&mut s3, 2);

    let mut s4 = setup_make_v2(amount_to_receive, amount_to_give);
    let cancel_v2 = do_cancel(&mut s4, 5);

    let sep = "+-------------+----------+----------+-------+";
    println!("{sep}");
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>5} |",
        "instruction", "unsafe", "wincode", "diff"
    );
    println!("{sep}");
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "make",
        make_v1,
        make_v2,
        make_v2 as i64 - make_v1 as i64
    );
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "take",
        take_v1,
        take_v2,
        take_v2 as i64 - take_v1 as i64
    );
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "cancel",
        cancel_v1,
        cancel_v2,
        cancel_v2 as i64 - cancel_v1 as i64
    );
    println!("{sep}");
}
