use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::invoke_signed,
    },
};
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token, TokenAccount},
};

use crate::constants::{
    AUTO_REFUND_DISCRIMINATOR, ESCROW_SEED, QUEUE_AUTHORITY_SEED, QUEUE_TASK_DISCRIMINATOR,
    TUKTUK_PROGRAM_ID,
};
use crate::state::Escrow;

#[derive(AnchorSerialize)]
enum TriggerV0 {
    Now,
    Timestamp(i64),
}

#[derive(AnchorSerialize)]
struct CompiledInstructionV0 {
    program_id_index: u8,
    accounts: Vec<u8>,
    data: Vec<u8>,
}

#[derive(AnchorSerialize)]
struct CompiledTransactionV0 {
    num_rw_signers: u8,
    num_ro_signers: u8,
    num_rw: u8,
    accounts: Vec<Pubkey>,
    instructions: Vec<CompiledInstructionV0>,
    signer_seeds: Vec<Vec<Vec<u8>>>,
}

#[derive(AnchorSerialize)]
enum TransactionSourceV0 {
    CompiledV0(CompiledTransactionV0),
}

#[derive(AnchorSerialize)]
struct QueueTaskArgsV0 {
    id: u16,
    trigger: TriggerV0,
    transaction: TransactionSourceV0,
    crank_reward: Option<u64>,
    free_tasks: u8,
    description: String,
}

#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(
        has_one = mint_a,
        has_one = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: Passed through to TukTuk CPI
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,

    /// CHECK: Derived and verified by TukTuk program
    #[account(mut)]
    pub task_queue_authority: UncheckedAccount<'info>,

    /// CHECK: Initialized in CPI - address = PDA(["task", task_queue, task_id], tuktuk)
    #[account(mut)]
    pub task: UncheckedAccount<'info>,

    /// CHECK: PDA signer - no data stored here
    #[account(
        mut,
        seeds = [QUEUE_AUTHORITY_SEED],
        bump,
    )]
    pub queue_authority: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: TukTuk program
    #[account(address = Pubkey::from_str(TUKTUK_PROGRAM_ID).unwrap())]
    pub tuktuk_program: UncheckedAccount<'info>,
}

impl<'info> Schedule<'info> {
    pub fn schedule(&self, task_id: u16, bumps: &ScheduleBumps) -> Result<()> {
        let maker_key = self.maker.key();
        let mint_a_key = self.mint_a.key();
        let escrow_key = self.escrow.key();
        let vault_key = self.vault.key();
        let maker_ata_a_key = get_associated_token_address(&maker_key, &mint_a_key);
        let token_program_key = self.token_program.key();
        let system_program_key = self.system_program.key();
        let our_program_key = crate::ID;

        let compiled_tx = CompiledTransactionV0 {
            num_rw_signers: 0,
            num_ro_signers: 0,
            num_rw: 4,
            accounts: vec![
                maker_key,
                maker_ata_a_key,
                escrow_key,
                vault_key,
                mint_a_key,
                token_program_key,
                system_program_key,
                our_program_key,
            ],
            instructions: vec![CompiledInstructionV0 {
                program_id_index: 7,
                accounts: vec![0, 4, 1, 2, 3, 5, 6],
                data: AUTO_REFUND_DISCRIMINATOR.to_vec(),
            }],
            signer_seeds: vec![],
        };

        let args = QueueTaskArgsV0 {
            id: task_id,
            trigger: TriggerV0::Timestamp(self.escrow.expires_at),
            transaction: TransactionSourceV0::CompiledV0(compiled_tx),
            crank_reward: Some(5_000_000), // 0.005 SOL reward for the cranker
            free_tasks: 0,
            description: "escrow auto_refund on expiry".to_string(),
        };

        let mut ix_data = QUEUE_TASK_DISCRIMINATOR.to_vec();
        args.serialize(&mut ix_data)?;

        let tuktuk_program_id = self.tuktuk_program.key();

        let ix = Instruction {
            program_id: tuktuk_program_id,
            accounts: vec![
                AccountMeta::new(self.maker.key(), true),
                AccountMeta::new_readonly(self.queue_authority.key(), true),
                AccountMeta::new_readonly(self.task_queue_authority.key(), false),
                AccountMeta::new(self.task_queue.key(), false),
                AccountMeta::new(self.task.key(), false),
                AccountMeta::new_readonly(self.system_program.key(), false),
            ],
            data: ix_data,
        };

        let queue_auth_seeds: &[&[&[u8]]] = &[&[QUEUE_AUTHORITY_SEED, &[bumps.queue_authority]]];

        invoke_signed(
            &ix,
            &[
                self.maker.to_account_info(),
                self.queue_authority.to_account_info(),
                self.task_queue_authority.to_account_info(),
                self.task_queue.to_account_info(),
                self.task.to_account_info(),
                self.system_program.to_account_info(),
            ],
            queue_auth_seeds,
        )?;

        msg!(
            "Scheduled auto_refund for escrow {} at timestamp {}",
            escrow_key,
            self.escrow.expires_at
        );

        Ok(())
    }
}
