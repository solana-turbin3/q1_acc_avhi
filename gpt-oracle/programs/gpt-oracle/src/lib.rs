pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("EWTRkd34BihiCCWW5Xtr1ff3RZjub8Q5TQ8Khb5cBbXJ");

#[program]
pub mod gpt_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, text: String) -> Result<()> {
        ctx.accounts.create_llm_context(text, &ctx.bumps)?;

        Ok(())
    }

    pub fn interact_with_llm(ctx: Context<Interact>, text: String) -> Result<()> {
        ctx.accounts.interact_with_llm(text)?;
        Ok(())
    }

    pub fn callback_from_llm(ctx: Context<Callback>, response: String) -> Result<()> {
        ctx.accounts.callback_from_llm(response)?;
        Ok(())
    }
}
