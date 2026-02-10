use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct CloseUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        close = user,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Box<Account<'info, UserAccount>>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseUser<'info> {
    
    pub fn close(&mut self) -> Result<()> {
        // All the closing logic is handled by the `close` constraint in the Accounts struct
        Ok(())
    }
}