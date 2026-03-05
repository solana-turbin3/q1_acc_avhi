use anchor_lang::prelude::*;

use crate::state::{ExternalValidationResult, Oracle, OracleValidation};

#[derive(Accounts)]
pub struct InitOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = Oracle::INIT_SPACE,
        seeds = [b"oracle"],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: lamport vault PDA
    #[account(
        mut,
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump
    )]
    pub reward_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl InitOracle<'_> {
    pub fn init_oracle(&mut self, bumps: &InitOracleBumps) -> Result<()> {
        self.oracle.validation = OracleValidation::V1 {
            create: ExternalValidationResult::Pass,
            transfer: ExternalValidationResult::Rejected,
            burn: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
        };
        self.oracle.bump = bumps.oracle;
        self.oracle.vault_bump = bumps.reward_vault;
        Ok(())
    }
}
