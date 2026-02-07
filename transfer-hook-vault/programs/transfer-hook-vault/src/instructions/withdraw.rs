use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::{
    error::ErrorCode, UserAccount, Vault, EXTRA_ACCOUNT_METAS, VAULT_CONFIG, WHITELIST_ENTRY,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
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

    #[account(address = vault.mint)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Vault PDA's whitelist entry (vault is source, so hook checks its whitelist)
    #[account(
        seeds = [WHITELIST_ENTRY.as_bytes(), vault.key().as_ref()],
        bump = vault_whitelist.bump,
    )]
    pub vault_whitelist: Account<'info, UserAccount>,

    /// CHECK: ExtraAccountMetaList PDA
    #[account(
        seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    /// CHECK: Our program - Token-2022 needs it to CPI the hook
    #[account(address = crate::ID)]
    pub hook_program: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(
            self.user_account.amount >= amount,
            ErrorCode::InsufficientFunds
        );

        let admin_key = self.vault.admin.key();
        let signer_seeds: &[&[u8]] = &[
            VAULT_CONFIG.as_bytes(),
            admin_key.as_ref(),
            &[self.vault.bump],
        ];

        let binding = [signer_seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.vault_token_account.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.vault.to_account_info(),
                mint: self.mint.to_account_info(),
            },
            &binding,
        )
        .with_remaining_accounts(vec![
            self.extra_account_meta_list.to_account_info(),
            self.vault_whitelist.to_account_info(),
            self.hook_program.to_account_info(),
        ]);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        self.user_account.amount = self
            .user_account
            .amount
            .checked_sub(amount)
            .ok_or(ErrorCode::InsufficientFunds)?;

        msg!("Withdrew {} tokens", amount);
        Ok(())
    }
}
