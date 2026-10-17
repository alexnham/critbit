use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;
use errors::*;
use events::*;
pub use instructions::*;
use state::*;
declare_id!("BgT4LsMdYp7q73t2UtkM2soKgXJHdXo2joQoaAM9UFxN");
#[program]
pub mod critbit_staking {
    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, duration_seconds: u64) -> Result<()> {
        instructions::initialize_pool::handle_initialize_pool(ctx, duration_seconds)
    }
    pub fn register_character(ctx: Context<RegisterCharacter>, character_id: u32) -> Result<()> {
        instructions::register_character::handle_register_character(ctx, character_id)
    }
    pub fn open_position(ctx: Context<OpenPosition>) -> Result<()> {
        instructions::open_position::handle_open_position(ctx)
    }
    pub fn fund_rewards(ctx: Context<FundRewards>, amount: u64) -> Result<()> {
        instructions::fund_rewards::handle_fund_rewards(ctx, amount)
    }
    pub fn stake(ctx: Context<ManagePosition>, amount: u64) -> Result<()> {
        instructions::stake::handle_stake(ctx, amount)
    }
    pub fn unstake(ctx: Context<ManagePosition>, amount: u64) -> Result<()> {
        instructions::unstake::handle_unstake(ctx, amount)
    }
    pub fn claim(ctx: Context<ManagePosition>) -> Result<()> {
        instructions::claim::handle_claim(ctx)
    }
    pub fn reclaim_unused_rewards(ctx: Context<ReclaimUnusedRewards>, amount: u64) -> Result<()> {
        instructions::reclaim_unused_rewards::handle_reclaim_unused_rewards(ctx, amount)
    }
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handle_close_position(ctx)
    }
}
fn transfer<'info>(
    program: &Program<'info, Token>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: &Account<'info, Mint>,
    amount: u64,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    token::transfer_checked(
        CpiContext::new(
            program.key(),
            TransferChecked {
                from,
                to,
                authority,
                mint: mint.to_account_info(),
            },
        )
        .with_signer(seeds),
        amount,
        mint.decimals,
    )
}
