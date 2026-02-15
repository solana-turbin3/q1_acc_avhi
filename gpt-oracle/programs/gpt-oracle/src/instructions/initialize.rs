use anchor_lang::prelude::*;
use solana_gpt_oracle::{cpi::create_llm_context, Counter};

use crate::Agent;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 1,
        seeds = [b"agent", payer.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(mut)]
    pub counter: Account<'info, Counter>,

    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,

    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn create_llm_context(&mut self, text: String, bumps: &InitializeBumps) -> Result<()> {
        self.agent.set_inner(Agent {
            context: self.llm_context.key(),
            bump: bumps.agent,
        });

        let cpi_program = self.oracle_program.to_account_info();
        let cpi_acc = solana_gpt_oracle::cpi::accounts::CreateLlmContext {
            payer: self.payer.to_account_info(),
            counter: self.counter.to_account_info(),
            context_account: self.llm_context.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_acc);
        create_llm_context(cpi_ctx, text)?;

        Ok(())
    }
}
