pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8iCfkBsLg5G7HkJZt95KnutdyqnpFuUdAskwMjiupDzu");

#[program]
pub mod pyth_scheduler {
    use super::*;

    pub fn update_price(ctx: Context<UpdatePrice>) -> Result<()> {
        ctx.accounts.update_price(&ctx.bumps)
    }

    pub fn schedule(ctx: Context<Schedule>, task_id: u16) -> Result<()> {
        ctx.accounts.schedule(task_id, &ctx.bumps)
    }
}
