#[cfg(test)]
mod test {

    use {
        anchor_lang::{
            prelude::msg, solana_program::program_pack::Pack, AccountDeserialize, InstructionData,
            ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo},
        solana_account::Account,
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_rpc_client::rpc_client::RpcClient,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
    };

    use anchor_lang::prelude::Pubkey;
    use solana_clock::Clock;

    static PROGRAM_ID: Pubkey = crate::ID;

    fn pubkey_to_addr(pk: &Pubkey) -> Address {
        Address::from(pk.to_bytes())
    }

    fn addr_to_pubkey(addr: &Address) -> Pubkey {
        Pubkey::new_from_array(addr.to_bytes())
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        let _ = program.add_program(pubkey_to_addr(&PROGRAM_ID), &program_data);

        let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        let account_address =
            Pubkey::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        let fetched_account = rpc_client
            .get_account(&account_address)
            .expect("Failed to fetch account from devnet");

        program
            .set_account(
                payer.pubkey(),
                Account {
                    lamports: fetched_account.lamports,
                    data: fetched_account.data,
                    owner: Address::from(fetched_account.owner.to_bytes()),
                    executable: fetched_account.executable,
                    rent_epoch: fetched_account.rent_epoch,
                },
            )
            .unwrap();

        msg!("Lamports of fetched account: {}", fetched_account.lamports);

        (program, payer)
    }

    #[test]
    fn test_make() {
        let (mut program, payer) = setup();

        let maker = payer.pubkey();

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

        let maker_pubkey = addr_to_pubkey(&maker);
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker_pubkey.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        let vault =
            associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));
        msg!("Vault PDA: {}\n", vault);

        let asspciated_token_program = associated_token::spl_associated_token_account::ID;
        let token_program = spl_token::ID;
        let system_program = anchor_lang::system_program::ID;

        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let anchor_accounts = crate::accounts::Make {
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

        let tx = program.send_transaction(transaction).unwrap();

        msg!("\n\nMake transaction successfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let vault_account = program.get_account(&pubkey_to_addr(&vault)).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, addr_to_pubkey(&mint_a));

        let escrow_account = program.get_account(&pubkey_to_addr(&escrow)).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker_pubkey);
        assert_eq!(escrow_data.mint_a, addr_to_pubkey(&mint_a));
        assert_eq!(escrow_data.mint_b, addr_to_pubkey(&mint_b));
        assert_eq!(escrow_data.receive, 10);
    }

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
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker_pubkey.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        let vault =
            associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));
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

        clock.unix_timestamp += 5 * 24 * 60 * 60; // 5 days

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
        let taker_ata_a_data =
            spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_ata_a_data.amount, 10);
        assert_eq!(taker_ata_a_data.owner, addr_to_pubkey(&taker.pubkey()));
        assert_eq!(taker_ata_a_data.mint, addr_to_pubkey(&mint_a));

        let maker_ata_b_account = program
            .get_account(&pubkey_to_addr(&addr_to_pubkey(&maker_ata_b)))
            .unwrap();
        let maker_ata_b_data =
            spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
        assert_eq!(maker_ata_b_data.amount, 10);
        assert_eq!(maker_ata_b_data.owner, addr_to_pubkey(&maker));
        assert_eq!(maker_ata_b_data.mint, addr_to_pubkey(&mint_b));

        let taker_ata_b_account = program
            .get_account(&pubkey_to_addr(&addr_to_pubkey(&taker_ata_b)))
            .unwrap();
        let taker_ata_b_data =
            spl_token::state::Account::unpack(&taker_ata_b_account.data).unwrap();
        assert_eq!(taker_ata_b_data.amount, 1000000000 - 10);

        let vault_account = program.get_account(&pubkey_to_addr(&vault));
        assert!(vault_account.is_none(), "Vault should be closed");

        let escrow_account = program.get_account(&pubkey_to_addr(&escrow));
        assert!(escrow_account.is_none(), "Escrow should be closed");
    }

    #[test]
    fn test_refund() {
        let (mut program, payer) = setup();

        let maker = payer.pubkey();

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

        let maker_pubkey = addr_to_pubkey(&maker);
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker_pubkey.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        let vault =
            associated_token::get_associated_token_address(&escrow, &addr_to_pubkey(&mint_a));
        msg!("Vault PDA: {}\n", vault);

        let asspciated_token_program = associated_token::spl_associated_token_account::ID;
        let token_program = spl_token::ID;
        let system_program = anchor_lang::system_program::ID;

        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

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

        let refund_accounts = crate::accounts::Refund {
            maker: maker_pubkey,
            mint_a: addr_to_pubkey(&mint_a),
            maker_ata_a: addr_to_pubkey(&maker_ata_a),
            escrow,
            vault,
            token_program,
            system_program,
        }
        .to_account_metas(None);

        let refund_ix = Instruction {
            program_id: pubkey_to_addr(&PROGRAM_ID),
            accounts: refund_accounts
                .into_iter()
                .map(|m| solana_instruction::AccountMeta {
                    pubkey: pubkey_to_addr(&m.pubkey),
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data: crate::instruction::Refund.data(),
        };

        let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        let tx = program.send_transaction(transaction).unwrap();

        msg!("\n\nRefund transaction successfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_ata_a_account = program
            .get_account(&pubkey_to_addr(&addr_to_pubkey(&maker_ata_a)))
            .unwrap();
        let maker_ata_a_data =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
        assert_eq!(maker_ata_a_data.amount, 1000000000);
        assert_eq!(maker_ata_a_data.owner, addr_to_pubkey(&maker));
        assert_eq!(maker_ata_a_data.mint, addr_to_pubkey(&mint_a));

        let vault_account = program.get_account(&pubkey_to_addr(&vault));
        assert!(vault_account.is_none(), "Vault should be closed");

        let escrow_account = program.get_account(&pubkey_to_addr(&escrow));
        assert!(escrow_account.is_none(), "Escrow should be closed");
    }
}
