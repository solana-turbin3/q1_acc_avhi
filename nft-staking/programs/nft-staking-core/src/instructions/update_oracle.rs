use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    errors::StakingError,
    helpers::{is_allowed, is_correct_time, REWARD_LAMPORTS},
    state::{ExternalValidationResult, Oracle, OracleValidation},
};

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump = oracle.vault_bump,
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateOracle<'info> {
    pub fn update_oracle(&mut self) -> Result<()> {
        match is_allowed(Clock::get()?.unix_timestamp) {
            true => {
                require!(
                    self.oracle.validation
                        == OracleValidation::V1 {
                            transfer: ExternalValidationResult::Rejected,
                            create: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass,
                        },
                    StakingError::AlreadyUpdated
                );

                self.oracle.validation = OracleValidation::V1 {
                    transfer: ExternalValidationResult::Approved,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass,
                };
            }
            false => {
                require!(
                    self.oracle.validation
                        == OracleValidation::V1 {
                            transfer: ExternalValidationResult::Approved,
                            create: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass,
                        },
                    StakingError::AlreadyUpdated
                );

                self.oracle.validation = OracleValidation::V1 {
                    transfer: ExternalValidationResult::Rejected,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass,
                };
            }
        }

        let reward_vault_lamports = self.reward_vault.lamports();
        let oracle_key = self.oracle.key();
        let signer_seeds: &[&[u8]] = &[
            b"reward_vault",
            oracle_key.as_ref(),
            &[self.oracle.vault_bump],
        ];

        if is_correct_time(Clock::get()?.unix_timestamp) && reward_vault_lamports > REWARD_LAMPORTS
        {
            transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    Transfer {
                        from: self.reward_vault.to_account_info(),
                        to: self.payer.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                REWARD_LAMPORTS,
            )?;
        }

        Ok(())
    }
}

/// Anyone can deposit SOL into the reward vault to fund crank rewards.
#[derive(Accounts)]
pub struct FundVault<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump = oracle.vault_bump,
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl FundVault<'_> {
    pub fn fund_vault(&mut self, amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.funder.to_account_info(),
                    to: self.reward_vault.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}
