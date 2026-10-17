use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub config: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub protocol_fees_a: u64,
    pub protocol_fees_b: u64,
    pub bump: u8,
}
