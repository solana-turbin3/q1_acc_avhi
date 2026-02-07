use anchor_lang::prelude::*;

use crate::{error::ErrorCode, UserAccount, Vault, WHITELIST_ENTRY};

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin @ ErrorCode::Unauthorized,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
         close = admin,
         seeds = [WHITELIST_ENTRY.as_bytes(), address.as_ref()],
        bump,
     )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> RemoveUser<'info> {
    pub fn remove_user(&mut self, address: Pubkey) -> Result<()> {
        msg!("Removed {} from whitelist", address);
        Ok(())
    }
}
