use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid account size calculation")]
    InvalidAccountSize,
    #[msg("Failed to initialize extra account meta list")]
    InitializationFailed,
    #[msg("Error creating extra account meta")]
    ExtraAccountMetaError,
    #[msg("Failed to initialize extension")]
    ExtensionInitializationFailed,
    #[msg("Unauthorized: Only admin can perform this action")]
    Unauthorized,
}
