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
    #[msg("Failed to create extra account meta")]
    ExtraAccountMetaError,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Arithmetic overflow")]
    Overflow,
}
