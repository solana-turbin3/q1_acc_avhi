use anchor_lang::prelude::*;
use anchor_spl::token_interface::{approve, Approve, TokenAccount, TokenInterface};

use crate::{error::ErrorCode, UserAccount, Vault, VAULT_CONFIG, WHITELIST_ENTRY};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    #[account(
        mut,
        token::mint = vault.mint,
        token::authority = vault,
        token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(
            self.user_account.amount >= amount,
            ErrorCode::InsufficientFunds
        );

        // Approve user as delegate on vault's token account.
        // Client must follow this with a transfer_checked ix in the same tx.
        // Since the user is the delegate authority, the transfer hook
        // checks the user's whitelist — no need to whitelist the vault PDA.
        let admin_key = self.vault.admin;
        let bump = self.vault.bump;
        let signer_seeds: &[&[&[u8]]] = &[&[VAULT_CONFIG.as_bytes(), admin_key.as_ref(), &[bump]]];

        approve(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Approve {
                    to: self.vault_token_account.to_account_info(),
                    delegate: self.user.to_account_info(),
                    authority: self.vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        self.user_account.amount = self
            .user_account
            .amount
            .checked_sub(amount)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Approved withdrawal of {} tokens", amount);
        Ok(())
    }
}
