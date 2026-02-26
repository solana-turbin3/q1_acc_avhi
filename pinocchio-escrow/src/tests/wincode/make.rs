use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::super::helpers::{load_svm, program_id, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID};

#[test]
fn test_make() {
    let (mut svm, maker) = load_svm();

    let mint_a = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    let amount_to_receive = 100_000_000u64;
    let amount_to_give = 500_000_000u64;

    let (escrow_pda, bump) =
        Pubkey::find_program_address(&[b"escrow", maker.pubkey().as_ref()], &program_id());

    let escrow_ata =
        spl_associated_token_account::get_associated_token_address(&escrow_pda, &mint_a);

    MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, amount_to_give)
        .send()
        .unwrap();

    let data = [
        vec![3u8],
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
        vec![bump],
    ]
    .concat();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(escrow_ata, false),
            AccountMeta::new(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(
                ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap(),
                false,
            ),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let blockhash = svm.latest_blockhash();
    let tx = svm
        .send_transaction(Transaction::new(&[&maker], msg, blockhash))
        .unwrap();

    println!("{:<12} | {:>6} CUs", "make v2", tx.compute_units_consumed);
}
