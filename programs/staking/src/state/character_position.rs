use crate::{
    errors::StakingError,
    math::rewards,
    state::{CharacterConfig, RewardPool},
};
use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace)]
pub struct CharacterPosition {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub character: Pubkey,
    pub staked_base_units: u64,
    pub level: u8,
    pub reward_rate_bps: u16,
    pub pending_rewards_base_units: u64,
    pub last_reward_timestamp: i64,
    /// Fractional reward numerator carried across claims and level changes.
    pub reward_remainder: u128,
    pub reserved_rewards: u64,
    pub bump: u8,
}
impl CharacterPosition {
    pub fn settle(&mut self, now: i64, end: i64) -> Result<()> {
        let until = now.min(end);
        require!(
            until >= self.last_reward_timestamp,
            StakingError::InvalidTime
        );
        let elapsed = u64::try_from(until - self.last_reward_timestamp)
            .map_err(|_| error!(StakingError::Math))?;
        let (earned, remainder) = rewards::accrue(
            self.staked_base_units,
            self.reward_rate_bps,
            elapsed,
            self.reward_remainder,
        )?;
        self.pending_rewards_base_units = self
            .pending_rewards_base_units
            .checked_add(earned)
            .ok_or(StakingError::Math)?;
        self.reward_remainder = remainder;
        self.last_reward_timestamp = until;
        Ok(())
    }
    /// Recompute the conservative obligation after settling and changing stake/claim.
    pub fn reserve(
        &mut self,
        pool: &mut RewardPool,
        character: &CharacterConfig,
        reward_balance: u64,
    ) -> Result<()> {
        (self.level, self.reward_rate_bps) = character.tier(self.staked_base_units);
        let seconds = u64::try_from(pool.end_timestamp - self.last_reward_timestamp)
            .map_err(|_| error!(StakingError::Math))?;
        let future = rewards::reserve(
            self.staked_base_units,
            self.reward_rate_bps,
            seconds,
            self.reward_remainder,
        )?;
        let new = self
            .pending_rewards_base_units
            .checked_add(future)
            .ok_or(StakingError::Math)?;
        let total = pool
            .reserved_rewards
            .checked_sub(self.reserved_rewards)
            .and_then(|n| n.checked_add(new))
            .ok_or(StakingError::Math)?;
        require!(total <= reward_balance, StakingError::InsufficientFunding);
        pool.reserved_rewards = total;
        self.reserved_rewards = new;
        Ok(())
    }
}
