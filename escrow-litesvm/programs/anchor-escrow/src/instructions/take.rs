use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer},
};

use crate::error::ErrorCode;
use crate::state::Escrow;

//Create context
#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub taker_ata_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub taker_ata_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub maker_ata_b: Account<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub clock: Sysvar<'info, Clock>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    pub fn validate(&self) -> Result<()> {
        let now = self.clock.unix_timestamp;

        const FIVE_DAYS: i64 = 5 * 24 * 60 * 60;
        // Validate taker_ata_a belongs to taker and uses mint_a
        require_keys_eq!(
            self.taker_ata_a.owner,
            self.taker.key(),
            ErrorCode::ConstraintTokenOwner
        );
        require_keys_eq!(
            self.taker_ata_a.mint,
            self.mint_a.key(),
            ErrorCode::ConstraintTokenMint
        );

        // Validate taker_ata_b belongs to taker and uses mint_b
        require_keys_eq!(
            self.taker_ata_b.owner,
            self.taker.key(),
            ErrorCode::ConstraintTokenOwner
        );
        require_keys_eq!(
            self.taker_ata_b.mint,
            self.mint_b.key(),
            ErrorCode::ConstraintTokenMint
        );

        // Validate maker_ata_b belongs to maker and uses mint_b
        require_keys_eq!(
            self.maker_ata_b.owner,
            self.maker.key(),
            ErrorCode::ConstraintTokenOwner
        );
        require_keys_eq!(
            self.maker_ata_b.mint,
            self.mint_b.key(),
            ErrorCode::ConstraintTokenMint
        );

        // Validate vault belongs to escrow PDA and uses mint_a
        require_keys_eq!(
            self.vault.owner,
            self.escrow.key(),
            ErrorCode::ConstraintTokenOwner
        );
        require_keys_eq!(
            self.vault.mint,
            self.mint_a.key(),
            ErrorCode::ConstraintTokenMint
        );

        require!(
            now >= self.escrow.created_at + FIVE_DAYS,
            ErrorCode::TooEarlyToTake,
        );

        Ok(())
    }

    pub fn deposit(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, self.escrow.receive)
    }

    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        transfer(cpi_context, self.vault.amount)?;

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        close_account(cpi_context)
    }
}
