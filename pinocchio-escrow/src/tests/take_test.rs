use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::helpers::{program_id, setup_make, TOKEN_PROGRAM_ID};

#[test]
fn test_take() {
    let amount_to_receive = 100_000_000u64;
    let amount_to_give = 500_000_000u64;

    let mut s = setup_make(amount_to_receive, amount_to_give);

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
        data: vec![1u8],
    };

    let msg = Message::new(&[ix], Some(&taker.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&taker], msg, blockhash))
        .unwrap();

    println!("Take OK | CUs: {}", tx.compute_units_consumed);
}
