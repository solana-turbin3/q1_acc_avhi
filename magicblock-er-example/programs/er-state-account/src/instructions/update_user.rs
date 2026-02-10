use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> UpdateUser<'info> {
    pub fn update(&mut self, new_data: u64) -> Result<()> {

        // Update the data field
        self.user_account.data = new_data;
        
        Ok(())
    }
}