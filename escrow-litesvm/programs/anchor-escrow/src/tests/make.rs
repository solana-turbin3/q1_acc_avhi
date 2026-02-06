use anchor_lang::{
    solana_program::program_pack::Pack, AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{
        self, spl_associated_token_account::solana_program::native_token::LAMPORTS_PER_SOL,
    },
    token::spl_token,
};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::constants::ESCROW_SEED;

use super::helper::{addr_to_pubkey, pubkey_to_addr, setup, PROGRAM_ID};

#[test]
fn test_make() {
    let (mut program, payer) = setup();

    let maker = payer.pubkey();
    let taker = Keypair::new();

    program
        .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to taker");

    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &taker)
        .decimals(6)
        .authority(&taker.pubkey())
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();

    let escrow = anchor_lang::prelude::Pubkey::find_program_address(
        &[ESCROW_SEED, maker.as_ref(), &123u64.to_le_bytes()],
        &PROGRAM_ID,
    )
    .0;

    let vault = associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));

    let asspciated_token_program = associated_token::spl_associated_token_account::ID;
    let token_program = spl_token::ID;
    let system_program = anchor_lang::system_program::ID;

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
        .send()
        .unwrap();

    let anchor_accounts = crate::accounts::Make {
        maker: addr_to_pubkey(&maker),
        mint_a: addr_to_pubkey(&mint_a),
        mint_b: addr_to_pubkey(&mint_b),
        maker_ata_a: addr_to_pubkey(&maker_ata_a),
        escrow,
        vault,
        associated_token_program: asspciated_token_program,
        token_program,
        system_program,
    }
    .to_account_metas(None);

    let make_ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: anchor_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: pubkey_to_addr(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: crate::instruction::Make {
            deposit: 10,
            seed: 123u64,
            receive: 10,
        }
        .data(),
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    program.send_transaction(transaction).unwrap();

    let vault_account = program.get_account(&pubkey_to_addr(&vault)).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
    assert_eq!(vault_data.amount, 10);
    assert_eq!(vault_data.owner, escrow);
    assert_eq!(vault_data.mint, addr_to_pubkey(&mint_a));

    let escrow_account = program.get_account(&pubkey_to_addr(&escrow)).unwrap();
    let escrow_data =
        crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_data.seed, 123u64);
    assert_eq!(escrow_data.maker, addr_to_pubkey(&maker));
    assert_eq!(escrow_data.mint_a, addr_to_pubkey(&mint_a));
    assert_eq!(escrow_data.mint_b, addr_to_pubkey(&mint_b));
    assert_eq!(escrow_data.receive, 10);
}
