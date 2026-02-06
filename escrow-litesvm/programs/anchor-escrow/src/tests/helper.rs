use anchor_lang::prelude::Pubkey;
use litesvm::LiteSVM;
use solana_address::Address;
use solana_keypair::Keypair;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use std::path::PathBuf;

pub static PROGRAM_ID: Pubkey = crate::ID;

pub fn pubkey_to_addr(pk: &Pubkey) -> Address {
    Address::from(pk.to_bytes())
}

pub fn addr_to_pubkey(addr: &Address) -> Pubkey {
    Pubkey::new_from_array(addr.to_bytes())
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut program = LiteSVM::new();
    let payer = Keypair::new();

    program
        .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to payer");

    let so_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

    let _ = program.add_program(pubkey_to_addr(&PROGRAM_ID), &program_data);

    (program, payer)
}
