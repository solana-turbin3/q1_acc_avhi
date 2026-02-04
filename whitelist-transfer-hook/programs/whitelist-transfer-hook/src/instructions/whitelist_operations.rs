use anchor_lang::prelude::*;

use crate::constants::{CONFIG_SEED, WHITELIST_ENTRY_SEED};
use crate::error::ErrorCode;
use crate::state::config::Config;
use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddToWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = Whitelist::LEN,
        seeds = [WHITELIST_ENTRY_SEED, address.as_ref()],
        bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveFromWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        close = admin,
        seeds = [WHITELIST_ENTRY_SEED, address.as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhiteList<'info> {
    pub fn add_to_whitelist(&mut self, address: Pubkey, bumps: &AddToWhiteListBumps) -> Result<()> {
        self.whitelist_entry.set_inner(Whitelist {
            address,
            bump: bumps.whitelist_entry,
        });

        msg!("Added {} to whitelist", address);
        Ok(())
    }
}

impl<'info> RemoveFromWhiteList<'info> {
    pub fn remove_from_whitelist(&mut self, address: Pubkey) -> Result<()> {
        msg!("Remove {} from whitelist", address);
        Ok(())
    }
}
