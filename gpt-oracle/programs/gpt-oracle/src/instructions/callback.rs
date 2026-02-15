use anchor_lang::prelude::*;
use solana_gpt_oracle::Identity;

#[derive(Accounts)]
pub struct Callback<'info> {
    pub identity: Account<'info, Identity>,
}

impl<'info> Callback<'info> {
    pub fn callback_from_llm(&mut self, response: String) -> Result<()> {
        if !self.identity.to_account_info().is_signer {
            return Err(ProgramError::InvalidAccountData.into());
        }
        msg!("Response: {:?}", response);
        Ok(())
    }
}
