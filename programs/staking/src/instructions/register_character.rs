use crate::*;
#[derive(Accounts)]
#[instruction(character_id: u32)]
pub struct RegisterCharacter<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(has_one = admin, has_one = mint)]
    pub pool: Box<Account<'info, RewardPool>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(init, payer = admin, space = 8 + CharacterConfig::INIT_SPACE, seeds = [b"character", pool.key().as_ref(), &character_id.to_le_bytes()], bump)]
    pub character: Box<Account<'info, CharacterConfig>>,
    pub system_program: Program<'info, System>,
}
pub fn handle_register_character(ctx: Context<RegisterCharacter>, character_id: u32) -> Result<()> {
    let scale = 10u64
        .checked_pow(ctx.accounts.mint.decimals.into())
        .ok_or(StakingError::Math)?;
    let mut tiers = [CharacterTier {
        level: 0,
        required_stake_base_units: 0,
        reward_rate_bps: 0,
    }; 3];
    for (i, &(level, tokens, rate)) in constants::CHARACTER_TIERS.iter().enumerate() {
        tiers[i] = CharacterTier {
            level,
            required_stake_base_units: tokens.checked_mul(scale).ok_or(StakingError::Math)?,
            reward_rate_bps: rate,
        };
    }
    ctx.accounts.character.set_inner(CharacterConfig {
        pool: ctx.accounts.pool.key(),
        character_id,
        tiers,
        bump: ctx.bumps.character,
    });
    Ok(())
}
