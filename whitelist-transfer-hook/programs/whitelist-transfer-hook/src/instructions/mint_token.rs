use anchor_lang::{
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::token_2022::{
    spl_token_2022::{extension::ExtensionType, state::Mint as Token2022Mint},
    Token2022,
};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: We will create and initialize this account manually
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, decimals: u8) -> Result<()> {
        // Calculate the space needed for mint with TransferHook extension
        let extension_types = vec![ExtensionType::TransferHook];
        let space = ExtensionType::try_calculate_account_len::<Token2022Mint>(&extension_types)
            .map_err(|_| error!(crate::error::ErrorCode::ExtensionInitializationFailed))?;

        msg!("Mint account space needed: {} bytes", space);

        // Calculate rent
        let lamports = Rent::get()?.minimum_balance(space);

        // Create the mint account via CPI to System Program
        create_account(
            CpiContext::new(
                self.system_program.to_account_info(),
                CreateAccount {
                    from: self.user.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports,
            space as u64,
            &self.token_program.key(),
        )?;

        msg!("Mint account created");

        // Initialize the TransferHook extension via CPI
        let init_hook_ix =
            anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::instruction::initialize(
                &self.token_program.key(),
                &self.mint.key(),
                Some(self.user.key()),
                Some(crate::ID),
            )?;

        anchor_lang::solana_program::program::invoke(
            &init_hook_ix,
            &[self.mint.to_account_info()],
        )?;

        msg!("Transfer hook extension initialized");

        // Initialize the base mint via CPI
        let init_mint_ix = anchor_spl::token_2022::spl_token_2022::instruction::initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.user.key(),
            Some(&self.user.key()),
            decimals,
        )?;

        anchor_lang::solana_program::program::invoke(
            &init_mint_ix,
            &[self.mint.to_account_info()],
        )?;

        msg!("Mint initialized successfully");
        msg!("Mint address: {}", self.mint.key());
        msg!("Transfer hook program: {}", crate::ID);
        msg!("Transfer hook authority: {}", self.user.key());

        Ok(())
    }
}
