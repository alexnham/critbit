use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct CharacterTier {
    pub level: u8,
    pub required_stake_base_units: u64,
    pub reward_rate_bps: u16,
}
#[account]
#[derive(InitSpace)]
pub struct CharacterConfig {
    pub pool: Pubkey,
    pub character_id: u32,
    /// Immutable after registration so existing reward promises cannot change.
    pub tiers: [CharacterTier; 3],
    pub bump: u8,
}
impl CharacterConfig {
    pub fn tier(&self, stake: u64) -> (u8, u16) {
        self.tiers
            .iter()
            .rev()
            .find(|t| stake >= t.required_stake_base_units)
            .map(|t| (t.level, t.reward_rate_bps))
            .unwrap_or((0, 0))
    }
}
