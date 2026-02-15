pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8d6wKSQNNoqSu98EgLn5ZotmJMZHq8cgcfLGsiubUqZe");

#[program]
pub mod gpt_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.create_llm_context(&ctx.bumps)?;

        Ok(())
    }

    pub fn interact_with_llm(ctx: Context<Interact>) -> Result<()> {
        ctx.accounts.interact_with_llm()?;
        Ok(())
    }

    pub fn callback_from_llm(ctx: Context<Callback>, response: String) -> Result<()> {
        ctx.accounts.callback_from_llm(response)?;
        Ok(())
    }

    pub fn schedule(ctx: Context<Schedule>, task_id: u16) -> Result<()> {
        ctx.accounts.schedule(task_id, &ctx.bumps)
    }
}
