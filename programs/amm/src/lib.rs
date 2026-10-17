use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, TransferChecked};
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;
use errors::*;
use events::*;
pub use instructions::*;
use state::*;

declare_id!("2UJP3iqx5Pn6SWuZarFfhWcxYSUuvQPNbXmjArSrWqaA");
#[program]
pub mod critbit_amm {
    use super::*;
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        fee_bps: u16,
        protocol_share_bps: u16,
    ) -> Result<()> {
        instructions::initialize_config::handle_initialize_config(ctx, fee_bps, protocol_share_bps)
    }
    pub fn create_pool(
        ctx: Context<CreatePool>,
        amount_a: u64,
        amount_b: u64,
        min_shares: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::create_pool::handle_create_pool(ctx, amount_a, amount_b, min_shares, deadline)
    }
    pub fn add_liquidity(
        ctx: Context<Liquidity>,
        shares: u64,
        max_a: u64,
        max_b: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::add_liquidity::handle_add_liquidity(ctx, shares, max_a, max_b, deadline)
    }
    pub fn remove_liquidity(
        ctx: Context<Liquidity>,
        shares: u64,
        min_a: u64,
        min_b: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::remove_liquidity::handle_remove_liquidity(ctx, shares, min_a, min_b, deadline)
    }
    pub fn swap(
        ctx: Context<Swap>,
        a_to_b: bool,
        input: u64,
        min_output: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::swap::handle_swap(ctx, a_to_b, input, min_output, deadline)
    }
    pub fn collect_protocol_fees(ctx: Context<CollectProtocolFees>) -> Result<()> {
        instructions::collect_protocol_fees::handle_collect_protocol_fees(ctx)
    }
}
fn check_deadline(deadline: i64) -> Result<()> {
    require!(Clock::get()?.unix_timestamp <= deadline, AmmError::Expired);
    Ok(())
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
