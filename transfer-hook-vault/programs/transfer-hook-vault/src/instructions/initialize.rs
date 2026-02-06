use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;

use crate::Vault;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Vault::LEN,
        seeds = [b"vault_config", admin.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: We will create and initialize this account manually
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, decimal: u8, bump: InitializeBumps) -> Result<()> {
        self.vault.set_inner(Vault {
            admin: self.admin.key(),
            mint: self.mint.key(),
            bump: bump.vault,
        });
        Ok(())
    }
}
