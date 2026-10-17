use anchor_lang::prelude::*;
#[error_code]
pub enum StakingError {
    #[msg("Arithmetic overflow or invalid accounting")]
    Math,
    #[msg("Amount must be positive")]
    ZeroAmount,
    #[msg("Invalid reward duration or unsupported mint decimals")]
    InvalidConfiguration,
    #[msg("Reward campaign has ended")]
    CampaignEnded,
    #[msg("Insufficient unreserved reward funding")]
    InsufficientFunding,
    #[msg("Insufficient staked principal")]
    InsufficientStake,
    #[msg("Position still has principal or whole-token-base-unit rewards")]
    PositionNotEmpty,
    #[msg("Clock moved backwards")]
    InvalidTime,
}
