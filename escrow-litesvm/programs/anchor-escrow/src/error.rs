use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Tokens cannot be claimed yet")]
    TooEarlyToTake,

    #[msg("Invalid token mint provided")]
    ConstraintTokenMint,

    #[msg("Invalid token owner")]
    ConstraintTokenOwner,
}
