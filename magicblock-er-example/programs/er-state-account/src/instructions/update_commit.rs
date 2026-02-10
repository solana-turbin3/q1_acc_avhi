use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_accounts};

use crate::state::UserAccount;

#[commit]
#[derive(Accounts)]
pub struct UpdateCommit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> UpdateCommit<'info> {
    
    pub fn update_commit(&mut self, new_data: u64) -> Result<()> {

        // Update the data field
        self.user_account.data = new_data;

        commit_accounts(
            &self.user.to_account_info(), 
            vec![&self.user_account.to_account_info()], 
            &self.magic_context, 
            &self.magic_program
        )?;
        
        Ok(())
    }
}