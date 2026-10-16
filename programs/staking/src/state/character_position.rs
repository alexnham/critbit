//! Planned PDA: [b"position", reward_pool, owner, character_id].
//! Each position tracks stake allocated to one user's character.
//! The same deposited tokens must never back multiple character positions.
//! TODO: Add owner, pool, character references, PDA bump, and Anchor constraints.

pub struct CharacterPosition {
    pub staked_base_units: u64,
    /// Zero means locked; 1..=3 selects a configured character tier.
    pub level: u8,
    pub pending_rewards_base_units: u64,
    pub last_reward_timestamp: i64,
}
