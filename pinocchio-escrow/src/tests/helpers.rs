use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

pub fn load_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/sbpf-solana-solana/release/escrow.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    svm.add_program(program_id(), &program_data).unwrap();

    (svm, payer)
}

pub struct MakeSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub escrow_pda: Pubkey,
    pub escrow_ata: Pubkey,
    pub maker_ata_a: Pubkey,
}

pub fn setup_make(amount_to_receive: u64, amount_to_give: u64) -> MakeSetup {
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

    let (escrow_pda, bump) =
        Pubkey::find_program_address(&[b"escrow", maker.pubkey().as_ref()], &program_id());

    let escrow_ata =
        spl_associated_token_account::get_associated_token_address(&escrow_pda, &mint_a);

    MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, amount_to_give)
        .send()
        .unwrap();

    let data = [
        vec![0u8],
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
        vec![bump],
        vec![0u8; 7],
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
    svm.send_transaction(Transaction::new(&[&maker], msg, blockhash))
        .unwrap();

    MakeSetup {
        svm,
        maker,
        mint_a,
        mint_b,
        escrow_pda,
        escrow_ata,
        maker_ata_a,
    }
}
