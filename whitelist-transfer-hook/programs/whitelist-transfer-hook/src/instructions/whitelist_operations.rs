use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::state::config::Config;
use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct AddToWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = Whitelist::LEN,
        seeds = [b"whitelist-entry", authority.as_ref()],
        bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct RemoveFromWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist-entry", authority.as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhiteList<'info> {
    pub fn add_to_whitelist(
        &mut self,
        authority: Pubkey,
        bumps: &AddToWhiteListBumps,
    ) -> Result<()> {
        self.whitelist_entry.set_inner(Whitelist {
            authority,
            bump: bumps.whitelist_entry,
        });

        msg!("Added {} to whitelist", authority);
        Ok(())
    }
}

impl<'info> RemoveFromWhiteList<'info> {
    pub fn remove_from_whitelist(&mut self, authority: Pubkey) -> Result<()> {
        msg!("Remove {} from whitelist", authority);
        Ok(())
    }
}
