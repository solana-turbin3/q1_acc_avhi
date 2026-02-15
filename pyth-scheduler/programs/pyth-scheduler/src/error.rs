use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid price feed account")]
    InvalidPriceFeed,
    #[msg("Price is too old")]
    PriceTooOld,
}
