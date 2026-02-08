use anchor_lang::prelude::*;

use crate::{error::ErrorCode, UserAccount, Vault, VAULT_CONFIG, WHITELIST_ENTRY};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [VAULT_CONFIG.as_bytes(), vault.admin.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [WHITELIST_ENTRY.as_bytes(), user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // Only update the ledger. Client must pair this instruction
        // with a transfer_checked instruction in the same transaction
        // to actually move tokens into the vault.
        // We can't CPI into transfer_checked here because Token-2022
        // would CPI back into our transfer_hook, causing reentrancy.
        self.user_account.amount = self
            .user_account
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Recorded deposit of {} tokens", amount);
        Ok(())
    }
}
