use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid account size")]
    InvalidAccountSize,
    #[msg("Mint initialization failed")]
    MintInitializationFailed,
    #[msg("Extension initialization failed")]
    ExtensionInitializationFailed,
    #[msg("Unauthorized")]
    Unauthorized,
}
