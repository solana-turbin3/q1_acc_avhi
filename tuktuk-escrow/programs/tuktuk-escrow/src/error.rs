use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Escrow has not expired yet")]
    EscrowNotExpired,

    #[msg("Escrow has already expired")]
    EscrowExpired,

    #[msg("Tokens cannot be claimed before the lock period ends")]
    TooEarlyToTake,

    #[msg("Invalid token mint provided")]
    InvalidMint,

    #[msg("Invalid token owner")]
    InvalidOwner,
}
