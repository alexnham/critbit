use anchor_lang::prelude::*;
#[event]
pub struct LiquidityChanged {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub adding: bool,
    pub amount_a: u64,
    pub amount_b: u64,
    pub shares: u64,
}
#[event]
pub struct Swapped {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub a_to_b: bool,
    pub input: u64,
    pub output: u64,
    pub protocol_fee: u64,
}
