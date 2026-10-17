use anchor_lang::prelude::*;
#[error_code]
pub enum AmmError {
    #[msg("Arithmetic overflow or invalid reserve accounting")]
    Math,
    #[msg("Amount must be positive")]
    ZeroAmount,
    #[msg("Invalid fee configuration")]
    InvalidFee,
    #[msg("Mints must be distinct and sorted by public key")]
    InvalidMints,
    #[msg("Insufficient initial liquidity")]
    InsufficientLiquidity,
    #[msg("Slippage limit exceeded")]
    Slippage,
    #[msg("Transaction deadline expired")]
    Expired,
}
