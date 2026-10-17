use anchor_lang::prelude::*;
#[event]
pub struct CharacterStakeChanged {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub character: Pubkey,
    pub stake: u64,
    pub level: u8,
}
#[event]
pub struct RewardsClaimed {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub character: Pubkey,
    pub amount: u64,
}
