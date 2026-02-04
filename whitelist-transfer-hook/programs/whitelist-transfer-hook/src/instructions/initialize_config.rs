use anchor_lang::prelude::*;

use crate::state::Config;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeConfig<'info> {
    pub fn initialize_config(&mut self, bumps: &InitializeConfigBumps) -> Result<()> {
        self.config.set_inner(Config {
            admin: self.admin.key(),
            bump: bumps.config,
        });

        msg!("Config initialized. Admin: {}", self.admin.key());
        Ok(())
    }
}
