use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;

pub fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

pub fn load_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/sbpf-solana-solana/release/pinocchio_fundraiser.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    svm.add_program(program_id(), &program_data).unwrap();

    (svm, payer)
}

pub fn fundraiser_pda(maker: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &program_id())
}

pub fn contributor_pda(fundraiser: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"contributor", fundraiser.as_ref(), contributor.as_ref()],
        &program_id(),
    )
}

pub struct InitializeSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub mint: Pubkey,
    pub fundraiser_pda: Pubkey,
    pub vault: Keypair,
    pub init_cu: u64,
}

pub fn setup_initialize(amount_to_raise: u64, duration: u8) -> InitializeSetup {
    let (mut svm, maker) = load_svm();

    let mint = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let (fundraiser_pda, bump) = fundraiser_pda(&maker.pubkey());
    let vault = Keypair::new();

    let mut data = vec![0u8]; // discriminator 0 = initialize
    data.extend_from_slice(&amount_to_raise.to_le_bytes());
    data.push(duration);
    data.push(bump);
    data.extend_from_slice(&[0u8; 6]);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(fundraiser_pda, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(vault.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let blockhash = svm.latest_blockhash();
    let tx = svm
        .send_transaction(Transaction::new(&[&maker, &vault], msg, blockhash))
        .unwrap();

    InitializeSetup {
        svm,
        maker,
        mint,
        fundraiser_pda,
        vault,
        init_cu: tx.compute_units_consumed,
    }
}

pub struct ContributeSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub contributor: Keypair,
    pub mint: Pubkey,
    pub fundraiser_pda: Pubkey,
    pub vault: Keypair,
    pub contributor_ata: Pubkey,
    pub contributor_state_pda: Pubkey,
    pub contribute_cu: u64,
}

pub fn setup_contribute(
    amount_to_raise: u64,
    duration: u8,
    contribute_amount: u64,
) -> ContributeSetup {
    let mut s = setup_initialize(amount_to_raise, duration);

    let contributor = Keypair::new();
    s.svm
        .airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();

    let contributor_ata = CreateAssociatedTokenAccount::new(&mut s.svm, &contributor, &s.mint)
        .owner(&contributor.pubkey())
        .send()
        .unwrap();

    MintTo::new(
        &mut s.svm,
        &s.maker,
        &s.mint,
        &contributor_ata,
        contribute_amount,
    )
    .send()
    .unwrap();

    let (contributor_state_pda, bump) = contributor_pda(&s.fundraiser_pda, &contributor.pubkey());

    let mut data = vec![1u8]; // discriminator 1 = contribute
    data.extend_from_slice(&contribute_amount.to_le_bytes());
    data.push(bump);
    data.extend_from_slice(&[0u8; 7]);

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(s.fundraiser_pda, false),
            AccountMeta::new_readonly(s.mint, false),
            AccountMeta::new(s.vault.pubkey(), false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(contributor_state_pda, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&contributor.pubkey()));
    let blockhash = s.svm.latest_blockhash();
    let tx = s
        .svm
        .send_transaction(Transaction::new(&[&contributor], msg, blockhash))
        .unwrap();

    ContributeSetup {
        svm: s.svm,
        maker: s.maker,
        contributor,
        mint: s.mint,
        fundraiser_pda: s.fundraiser_pda,
        vault: s.vault,
        contributor_ata,
        contributor_state_pda,
        contribute_cu: tx.compute_units_consumed,
    }
}
