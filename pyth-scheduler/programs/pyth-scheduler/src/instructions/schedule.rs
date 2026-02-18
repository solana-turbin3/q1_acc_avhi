use anchor_lang::{
    prelude::{instruction::Instruction, *},
    InstructionData,
};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{cpi::queue_task_v0, program::Tuktuk},
    TransactionSourceV0, TriggerV0,
};

use crate::{PriceStore, PRICE, QUEUE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [PRICE.as_bytes()],
        bump = price_store.bump,
    )]
    pub price_store: Account<'info, PriceStore>,

    /// CHECK: Pyth PriceUpdateV2 account, passed through to compiled TukTuk task
    pub price_feed: UncheckedAccount<'info>,

    /// CHECK: Passed through to TukTuk CPI
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,

    /// CHECK: Derived and verified by TukTuk program
    #[account(mut)]
    pub task_queue_authority: UncheckedAccount<'info>,

    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,

    /// CHECK: PDA signer
    #[account(
        mut,
        seeds = [QUEUE_AUTHORITY_SEED],
        bump,
    )]
    pub queue_authority: AccountInfo<'info>,

    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
}

impl<'info> Schedule<'info> {
    pub fn schedule(&self, task_id: u16, bumps: &ScheduleBumps) -> Result<()> {
        let update_price_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(self.payer.key(), false),
                AccountMeta::new(self.price_store.key(), false),
                AccountMeta::new_readonly(self.price_feed.key(), false),
                AccountMeta::new_readonly(System::id(), false),
            ],
            data: crate::instruction::UpdatePrice {}.data(),
        };

        let (compiled_tx, _) = compile_transaction(vec![update_price_ix], vec![]).unwrap();

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                tuktuk_program::tuktuk::cpi::accounts::QueueTaskV0 {
                    payer: self.payer.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&[QUEUE_AUTHORITY_SEED, &[bumps.queue_authority]]],
            ),
            tuktuk_program::types::QueueTaskArgsV0 {
                id: task_id,
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(5_000_000),
                free_tasks: 0,
                description: "update_sol_usd_price".to_string(),
            },
        )?;

        msg!(
            "Scheduled SOL/USD price update via TukTuk, task_id: {}",
            task_id
        );

        Ok(())
    }
}
