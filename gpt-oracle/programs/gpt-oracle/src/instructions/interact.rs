use crate::instruction;
use anchor_lang::prelude::*;
use solana_gpt_oracle::{cpi::accounts::InteractWithLlm, ContextAccount};

use crate::{Agent, ID};

#[derive(Accounts)]
pub struct Interact<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Checked oracle id
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(
        seeds = [b"agent", payer.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(address= agent.context)]
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Interact<'info> {
    pub fn interact_with_llm(&mut self, text: String) -> Result<()> {
        let cpi_program = self.oracle_program.to_account_info();
        let cpi_acc = InteractWithLlm {
            payer: self.payer.to_account_info(),
            context_account: self.context_account.to_account_info(),
            interaction: self.interaction.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_acc);

        let desc = instruction::CallbackFromLlm::DISCRIMINATOR
            .try_into()
            .expect("Must be 8 bytes");

        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, text, ID, desc, None)?;
        Ok(())
    }
}
