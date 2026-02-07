use anchor_lang::prelude::*;

use crate::{UserAccount, WHITELIST_ENTRY};

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [WHITELIST_ENTRY.as_bytes(), address.key().as_ref()],
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
