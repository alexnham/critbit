use anchor_lang::prelude::*;
use critbit_amm::math::{
    constant_product::quote_swap,
    fees::{down, up},
    liquidity::{initial_shares, MINIMUM_LIQUIDITY},
};
use critbit_staking::{constants::*, math::rewards, state::*};

fn fixture() -> (RewardPool, CharacterConfig, CharacterPosition) {
    let pool = RewardPool {
        admin: Pubkey::new_unique(),
        mint: Pubkey::new_unique(),
        stake_vault: Pubkey::new_unique(),
        reward_vault: Pubkey::new_unique(),
        end_timestamp: SECONDS_PER_YEAR as i64,
        total_staked: 0,
        reserved_rewards: 0,
        bump: 0,
    };
    let character = CharacterConfig {
        pool: Pubkey::new_unique(),
        character_id: 1,
        tiers: CHARACTER_TIERS.map(|(level, amount, rate)| CharacterTier {
            level,
            required_stake_base_units: amount,
            reward_rate_bps: rate,
        }),
        bump: 0,
    };
    let position = CharacterPosition {
        pool: character.pool,
        owner: Pubkey::new_unique(),
        character: Pubkey::new_unique(),
        staked_base_units: 0,
        level: 0,
        reward_rate_bps: 0,
        pending_rewards_base_units: 0,
        last_reward_timestamp: 0,
        reward_remainder: 0,
        reserved_rewards: 0,
        bump: 0,
    };
    (pool, character, position)
}
#[test]
fn swap_preserves_product_after_protocol_fee_exclusion() {
    for ra in [1_000u64, 1_000_000, 5_000_000_000] {
        for rb in [2_000u64, 2_000_000, 9_000_000_000] {
            for amount in [1_000u64, 10_000, 1_000_000] {
                for fee in [0, 30, 100] {
                    let net = amount - up(amount, fee.into(), 10_000).unwrap();
                    if down(net, rb, ra + net).unwrap() == 0 {
                        assert!(quote_swap(amount, ra, rb, fee, 2_000).is_err());
                        continue;
                    }
                    let (out, protocol) = quote_swap(amount, ra, rb, fee, 2_000).unwrap();
                    assert!(
                        u128::from(ra + amount - protocol) * u128::from(rb - out)
                            >= u128::from(ra) * u128::from(rb)
                    );
                }
            }
        }
    }
}
#[test]
fn liquidity_rounding_cannot_extract_value() {
    assert_eq!(
        initial_shares(1_000_000, 4_000_000).unwrap(),
        2_000_000 - MINIMUM_LIQUIDITY
    );
    assert!(initial_shares(1_000, 1_000).is_err());
    for supply in [1_000_000u64, 1_000_003] {
        for shares in [1u64, 31, 1_000] {
            let reserve = 9_123_457;
            let deposit = up(shares, reserve, supply).unwrap();
            let withdrawal = down(shares, reserve + deposit, supply + shares).unwrap();
            assert!(withdrawal <= deposit);
        }
    }
    assert!(up(u64::MAX, u64::MAX, 1).is_err());
    assert!(quote_swap(1, 1000, 1000, 30, 2000).is_err());
    assert!(quote_swap(100, u64::MAX, 1000, 0, 0).is_err());
}
#[test]
fn tiers_use_total_allocated_stake() {
    let (_, c, _) = fixture();
    for (amount, level, rate) in [
        (999, 0, 0),
        (1000, 1, 500),
        (4999, 1, 500),
        (5000, 2, 750),
        (9999, 2, 750),
        (10000, 3, 1000),
        (20000, 3, 1000),
    ] {
        assert_eq!(c.tier(amount), (level, rate));
    }
}
#[test]
fn annual_rates_and_fractional_carry() {
    for (_, stake, rate) in CHARACTER_TIERS {
        let (earned, rem) = rewards::accrue(stake, rate, SECONDS_PER_YEAR, 0).unwrap();
        assert_eq!(earned, stake * u64::from(rate) / 10_000);
        assert_eq!(rem, 0);
        let mut accumulated = 0;
        let mut remainder = 0;
        for _ in 0..365 {
            let (n, r) = rewards::accrue(stake, rate, 86400, remainder).unwrap();
            accumulated += n;
            remainder = r;
        }
        assert_eq!((accumulated, remainder), (earned, rem));
    }
}
#[test]
fn upgrades_do_not_apply_retroactively_and_withdrawal_releases_only_future() {
    let (mut pool, c, mut p) = fixture();
    p.staked_base_units = 1_000;
    p.reserve(&mut pool, &c, 10_000).unwrap();
    assert_eq!(pool.reserved_rewards, 50);
    p.settle((SECONDS_PER_YEAR / 2) as i64, pool.end_timestamp)
        .unwrap();
    assert_eq!(p.pending_rewards_base_units, 25);
    p.staked_base_units = 10_000;
    p.reserve(&mut pool, &c, 10_000).unwrap();
    assert_eq!(pool.reserved_rewards, 525);
    p.settle((SECONDS_PER_YEAR * 3 / 4) as i64, pool.end_timestamp)
        .unwrap();
    assert_eq!(p.pending_rewards_base_units, 275);
    p.staked_base_units = 0;
    p.reserve(&mut pool, &c, 10_000).unwrap();
    assert_eq!(pool.reserved_rewards, 275);
    assert_eq!(p.level, 0);
    p.settle(pool.end_timestamp + 100, pool.end_timestamp)
        .unwrap();
    assert_eq!(p.pending_rewards_base_units, 275);
}
#[test]
fn reservations_cover_multiple_positions_and_unclaimed_rewards_after_end() {
    let (mut pool, c, mut first) = fixture();
    let (_, _, mut second) = fixture();
    first.staked_base_units = 10_000;
    first.reserve(&mut pool, &c, 1_050).unwrap();
    second.staked_base_units = 1_000;
    second.reserve(&mut pool, &c, 1_050).unwrap();
    assert_eq!(pool.reserved_rewards, 1_050);
    second.staked_base_units = 5_000;
    assert!(second.reserve(&mut pool, &c, 1_050).is_err());
    first
        .settle(pool.end_timestamp + 10, pool.end_timestamp)
        .unwrap();
    first.reserve(&mut pool, &c, 1_050).unwrap();
    assert_eq!(pool.reserved_rewards, 1_050);
    assert_eq!(first.pending_rewards_base_units, 1_000);
}
