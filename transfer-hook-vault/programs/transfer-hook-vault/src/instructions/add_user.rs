use anchor_lang::prelude::*;

use crate::{error::ErrorCode, UserAccount, Vault, WHITELIST_ENTRY};

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin @ ErrorCode::Unauthorized,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = admin,
        seeds = [WHITELIST_ENTRY.as_bytes(), address.as_ref()],
        bump,
        space = UserAccount::LEN,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddUser<'info> {
    pub fn add_user(&mut self, address: Pubkey, bump: &AddUserBumps) -> Result<()> {
        self.user_account.set_inner(UserAccount {
            account: address,
            amount: 0,
            bump: bump.user_account,
        });

        msg!("Added {} to whitelist", address);

        Ok(())
    }
}
