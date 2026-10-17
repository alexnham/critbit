use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace)]
pub struct RewardPool {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_vault: Pubkey,
    pub end_timestamp: i64,
    pub total_staked: u64,
    /// Sum of each position's accrued rewards and reserved future rewards.
    pub reserved_rewards: u64,
    pub bump: u8,
}
