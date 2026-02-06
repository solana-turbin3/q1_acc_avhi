use anchor_lang::{
    prelude::msg, solana_program::program_pack::Pack, InstructionData, ToAccountMetas,
};
use anchor_spl::{associated_token, token::spl_token};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_clock::Clock;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::constants::{ESCROW_SEED, FIVE_DAYS};

use super::helper::{addr_to_pubkey, pubkey_to_addr, setup, PROGRAM_ID};

#[test]
fn test_take() {
    let (mut program, payer) = setup();

    let mut payer_account = program.get_account(&payer.pubkey()).unwrap();
    payer_account.lamports += 10 * LAMPORTS_PER_SOL;
    program.set_account(payer.pubkey(), payer_account).unwrap();

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
    msg!("Mint A: {}\n", mint_a);

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}\n", mint_b);

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}\n", maker_ata_a);

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    msg!("Taker ATA B: {}\n", taker_ata_b);

    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000000000)
        .send()
        .unwrap();

    let maker_pubkey = addr_to_pubkey(&maker);
    let escrow = anchor_lang::prelude::Pubkey::find_program_address(
        &[ESCROW_SEED, maker_pubkey.as_ref(), &123u64.to_le_bytes()],
        &PROGRAM_ID,
    )
    .0;
    msg!("Escrow PDA: {}\n", escrow);

    let vault = associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));
    msg!("Vault PDA: {}\n", vault);

    let asspciated_token_program = associated_token::spl_associated_token_account::ID;
    let token_program = spl_token::ID;
    let system_program = anchor_lang::system_program::ID;

    let make_accounts = crate::accounts::Make {
        maker: maker_pubkey,
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
        accounts: make_accounts
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

    msg!("\n\nMake transaction successful");

    let mut clock: Clock = program.get_sysvar();

    clock.unix_timestamp += FIVE_DAYS;

    program.set_sysvar(&clock);

    let anchor_accounts = crate::accounts::Take {
        maker: addr_to_pubkey(&maker),
        taker: addr_to_pubkey(&taker.pubkey()),
        mint_a: addr_to_pubkey(&mint_a),
        mint_b: addr_to_pubkey(&mint_b),
        taker_ata_a: addr_to_pubkey(&taker_ata_a),
        taker_ata_b: addr_to_pubkey(&taker_ata_b),
        maker_ata_b: addr_to_pubkey(&maker_ata_b),
        escrow,
        vault,
        associated_token_program: asspciated_token_program,
        token_program,
        system_program,
        clock: anchor_lang::solana_program::sysvar::clock::ID,
    }
    .to_account_metas(None);

    let take_ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: anchor_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: pubkey_to_addr(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: crate::instruction::Take.data(),
    };

    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = program.latest_blockhash();

    let transaction = Transaction::new(&[&taker], message, recent_blockhash);

    let tx = program.send_transaction(transaction).unwrap();

    msg!("\n\nTake transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    let taker_ata_a_account = program.get_account(&taker_ata_a).unwrap();
    let taker_ata_a_data = spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
    assert_eq!(taker_ata_a_data.amount, 10);
    assert_eq!(taker_ata_a_data.owner, addr_to_pubkey(&taker.pubkey()));
    assert_eq!(taker_ata_a_data.mint, addr_to_pubkey(&mint_a));

    let maker_ata_b_account = program
        .get_account(&pubkey_to_addr(&addr_to_pubkey(&maker_ata_b)))
        .unwrap();
    let maker_ata_b_data = spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
    assert_eq!(maker_ata_b_data.amount, 10);
    assert_eq!(maker_ata_b_data.owner, addr_to_pubkey(&maker));
    assert_eq!(maker_ata_b_data.mint, addr_to_pubkey(&mint_b));

    let taker_ata_b_account = program
        .get_account(&pubkey_to_addr(&addr_to_pubkey(&taker_ata_b)))
        .unwrap();
    let taker_ata_b_data = spl_token::state::Account::unpack(&taker_ata_b_account.data).unwrap();
    assert_eq!(taker_ata_b_data.amount, 1000000000 - 10);

    let vault_account = program.get_account(&pubkey_to_addr(&vault));
    assert!(vault_account.is_none(), "Vault should be closed");

    let escrow_account = program.get_account(&pubkey_to_addr(&escrow));
    assert!(escrow_account.is_none(), "Escrow should be closed");
}

#[test]
fn test_take_too_early() {
    let (mut program, payer) = setup();

    let mut payer_account = program.get_account(&payer.pubkey()).unwrap();
    payer_account.lamports += 10 * LAMPORTS_PER_SOL;
    program.set_account(payer.pubkey(), payer_account).unwrap();

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

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000000000)
        .send()
        .unwrap();

    let maker_pubkey = addr_to_pubkey(&maker);
    let escrow = anchor_lang::prelude::Pubkey::find_program_address(
        &[ESCROW_SEED, maker_pubkey.as_ref(), &123u64.to_le_bytes()],
        &PROGRAM_ID,
    )
    .0;

    let vault = associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));

    let asspciated_token_program = associated_token::spl_associated_token_account::ID;
    let token_program = spl_token::ID;
    let system_program = anchor_lang::system_program::ID;

    let make_accounts = crate::accounts::Make {
        maker: maker_pubkey,
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
        accounts: make_accounts
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

    msg!("\n\nMake transaction successful");

    // NOTE: We do NOT advance the clock here, so take should fail

    let anchor_accounts = crate::accounts::Take {
        maker: addr_to_pubkey(&maker),
        taker: addr_to_pubkey(&taker.pubkey()),
        mint_a: addr_to_pubkey(&mint_a),
        mint_b: addr_to_pubkey(&mint_b),
        taker_ata_a: addr_to_pubkey(&taker_ata_a),
        taker_ata_b: addr_to_pubkey(&taker_ata_b),
        maker_ata_b: addr_to_pubkey(&maker_ata_b),
        escrow,
        vault,
        associated_token_program: asspciated_token_program,
        token_program,
        system_program,
        clock: anchor_lang::solana_program::sysvar::clock::ID,
    }
    .to_account_metas(None);

    let take_ix = Instruction {
        program_id: pubkey_to_addr(&PROGRAM_ID),
        accounts: anchor_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: pubkey_to_addr(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: crate::instruction::Take.data(),
    };

    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = program.latest_blockhash();

    let transaction = Transaction::new(&[&taker], message, recent_blockhash);

    let result = program.send_transaction(transaction);

    assert!(
        result.is_err(),
        "Take should fail when called before 5 days"
    );
    msg!("\n\nTake correctly failed - too early to take");
}
