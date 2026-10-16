//! Planned PDA: [b"character", reward_pool, character_id].
//! Defines an available Critbit character and its staking tiers.
//! TODO: Add Anchor account fields and authorization constraints.

pub struct CharacterTier {
    pub level: u8,
    pub required_stake_base_units: u64,
    pub reward_rate_bps: u16,
}

pub struct CharacterConfig {
    pub character_id: u32,
    pub enabled: bool,
    pub tiers: [CharacterTier; 3],
}
