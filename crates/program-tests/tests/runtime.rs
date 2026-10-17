//! Execute compiled SBF programs in an isolated VM; never connects to an RPC.
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use critbit_amm as amm;
use critbit_staking as staking;
use litesvm::LiteSVM;
use solana_account::Account;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

fn vm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy");
    svm.add_program_from_file(amm::ID, root.join("critbit_amm.so"))
        .expect("run anchor build first");
    svm.add_program_from_file(staking::ID, root.join("critbit_staking.so"))
        .expect("run anchor build first");
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_000;
    svm.set_sysvar(&clock);
    svm
}
fn advance(svm: &mut LiteSVM, seconds: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp += seconds;
    svm.set_sysvar(&clock);
}
fn mint(svm: &mut LiteSVM, decimals: u8) -> Pubkey {
    let key = Pubkey::new_unique();
    let mut data = vec![0; Mint::LEN];
    Mint::pack(
        Mint {
            mint_authority: COption::None,
            supply: 1_000_000_000_000,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        key,
        Account {
            lamports: 10_000_000,
            data,
            owner: spl_token_interface::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    key
}
fn tokens(svm: &mut LiteSVM, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey {
    let key = Pubkey::new_unique();
    let mut data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(
        TokenAccount {
            mint,
            owner,
            amount,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        key,
        Account {
            lamports: 10_000_000,
            data,
            owner: spl_token_interface::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    key
}
fn balance(svm: &LiteSVM, key: Pubkey) -> u64 {
    TokenAccount::unpack(&svm.get_account(&key).unwrap().data)
        .unwrap()
        .amount
}
fn state<T: AccountDeserialize>(svm: &LiteSVM, key: Pubkey) -> T {
    T::try_deserialize(&mut svm.get_account(&key).unwrap().data.as_slice()).unwrap()
}
fn ix(id: Pubkey, accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: id,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}
fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> std::result::Result<(), String> {
    svm.expire_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}
fn pda(seeds: &[&[u8]], id: Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, &id).0
}

#[test]
#[ignore = "requires anchor build; run cargo test --test runtime -- --ignored"]
fn staking_unlock_upgrade_claim_reclaim_and_access_control() {
    let mut svm = vm();
    let user = Keypair::new();
    let owner = user.pubkey();
    svm.airdrop(&owner, 10_000_000_000).unwrap();
    let m = mint(&mut svm, 0);
    let user_tokens = tokens(&mut svm, m, owner, 100_000);
    let pool = pda(&[b"reward_pool", owner.as_ref(), m.as_ref()], staking::ID);
    let stake_vault = pda(&[b"stake_vault", pool.as_ref()], staking::ID);
    let reward_vault = pda(&[b"reward_vault", pool.as_ref()], staking::ID);
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            staking::accounts::InitializePool {
                admin: owner,
                mint: m,
                pool,
                stake_vault,
                reward_vault,
                token_program: spl_token_interface::ID,
                system_program: Pubkey::default(),
            },
            staking::instruction::InitializePool {
                duration_seconds: staking::constants::SECONDS_PER_YEAR,
            },
        ),
    )
    .unwrap();
    let character = pda(
        &[b"character", pool.as_ref(), &1u32.to_le_bytes()],
        staking::ID,
    );
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            staking::accounts::RegisterCharacter {
                admin: owner,
                pool,
                mint: m,
                character,
                system_program: Pubkey::default(),
            },
            staking::instruction::RegisterCharacter { character_id: 1 },
        ),
    )
    .unwrap();
    let position = pda(
        &[
            b"position",
            pool.as_ref(),
            owner.as_ref(),
            character.as_ref(),
        ],
        staking::ID,
    );
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            staking::accounts::OpenPosition {
                owner,
                pool,
                character,
                position,
                system_program: Pubkey::default(),
            },
            staking::instruction::OpenPosition {},
        ),
    )
    .unwrap();
    let manage = || staking::accounts::ManagePosition {
        owner,
        pool,
        mint: m,
        character,
        position,
        stake_vault,
        reward_vault,
        user_tokens,
        token_program: spl_token_interface::ID,
    };
    // Unfunded stake fails atomically.
    assert!(send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            manage(),
            staking::instruction::Stake { amount: 1_000 }
        )
    )
    .is_err());
    assert_eq!(balance(&svm, stake_vault), 0);
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            staking::accounts::FundRewards {
                funder: owner,
                pool,
                mint: m,
                source: user_tokens,
                reward_vault,
                token_program: spl_token_interface::ID,
            },
            staking::instruction::FundRewards { amount: 2_000 },
        ),
    )
    .unwrap();
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            manage(),
            staking::instruction::Stake { amount: 1_000 },
        ),
    )
    .unwrap();
    assert_eq!(
        state::<staking::state::CharacterPosition>(&svm, position).level,
        1
    );
    advance(&mut svm, (staking::constants::SECONDS_PER_YEAR / 2) as i64);
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            manage(),
            staking::instruction::Stake { amount: 9_000 },
        ),
    )
    .unwrap();
    let pos: staking::state::CharacterPosition = state(&svm, position);
    assert_eq!(
        (
            pos.level,
            pos.pending_rewards_base_units,
            pos.reserved_rewards
        ),
        (3, 25, 525)
    );
    // Admin cannot reclaim earned or promised rewards.
    let reclaim = || staking::accounts::ReclaimUnusedRewards {
        admin: owner,
        pool,
        mint: m,
        reward_vault,
        destination: user_tokens,
        token_program: spl_token_interface::ID,
    };
    assert!(send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            reclaim(),
            staking::instruction::ReclaimUnusedRewards { amount: 1_476 }
        )
    )
    .is_err());
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            reclaim(),
            staking::instruction::ReclaimUnusedRewards { amount: 1_475 },
        ),
    )
    .unwrap();
    send(
        &mut svm,
        &user,
        ix(staking::ID, manage(), staking::instruction::Claim {}),
    )
    .unwrap();
    assert_eq!(balance(&svm, stake_vault), 10_000);
    assert_eq!(balance(&svm, reward_vault), 500);
    // Forged owner and substituted vault cannot withdraw principal.
    let thief = Keypair::new();
    svm.airdrop(&thief.pubkey(), 1_000_000_000).unwrap();
    let thief_tokens = tokens(&mut svm, m, thief.pubkey(), 0);
    let mut forged = manage();
    forged.owner = thief.pubkey();
    forged.user_tokens = thief_tokens;
    assert!(send(
        &mut svm,
        &thief,
        ix(
            staking::ID,
            forged,
            staking::instruction::Unstake { amount: 1_000 }
        )
    )
    .is_err());
    let mut substituted = manage();
    substituted.stake_vault = reward_vault;
    assert!(send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            substituted,
            staking::instruction::Unstake { amount: 1_000 }
        )
    )
    .is_err());
    assert_eq!(balance(&svm, stake_vault), 10_000);
    advance(
        &mut svm,
        (staking::constants::SECONDS_PER_YEAR / 2) as i64 + 100,
    );
    assert!(send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            manage(),
            staking::instruction::Stake { amount: 1 }
        )
    )
    .is_err());
    assert!(send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            reclaim(),
            staking::instruction::ReclaimUnusedRewards { amount: 1 }
        )
    )
    .is_err());
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            manage(),
            staking::instruction::Unstake { amount: 10_000 },
        ),
    )
    .unwrap();
    assert_eq!(
        state::<staking::state::CharacterPosition>(&svm, position).level,
        0
    );
    assert_eq!(balance(&svm, stake_vault), 0);
    send(
        &mut svm,
        &user,
        ix(staking::ID, manage(), staking::instruction::Claim {}),
    )
    .unwrap();
    assert_eq!(balance(&svm, reward_vault), 0);
    assert_eq!(
        state::<staking::state::RewardPool>(&svm, pool).reserved_rewards,
        0
    );
    send(
        &mut svm,
        &user,
        ix(
            staking::ID,
            staking::accounts::ClosePosition {
                owner,
                pool,
                position,
            },
            staking::instruction::ClosePosition {},
        ),
    )
    .unwrap();
    assert!(svm.get_account(&position).is_none_or(|a| a.lamports == 0));
}

#[test]
#[ignore = "requires anchor build; run cargo test --test runtime -- --ignored"]
fn amm_liquidity_swaps_fees_and_slippage() {
    let mut svm = vm();
    let user = Keypair::new();
    let owner = user.pubkey();
    svm.airdrop(&owner, 10_000_000_000).unwrap();
    let m1 = mint(&mut svm, 6);
    let m2 = mint(&mut svm, 6);
    let (mint_a, mint_b) = if m1 < m2 { (m1, m2) } else { (m2, m1) };
    let user_a = tokens(&mut svm, mint_a, owner, 100_000_000);
    let user_b = tokens(&mut svm, mint_b, owner, 100_000_000);
    let config = pda(&[b"config", owner.as_ref()], amm::ID);
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            amm::accounts::InitializeConfig {
                admin: owner,
                config,
                system_program: Pubkey::default(),
            },
            amm::instruction::InitializeConfig {
                fee_bps: 30,
                protocol_share_bps: 2_000,
            },
        ),
    )
    .unwrap();
    let pool = pda(
        &[b"pool", config.as_ref(), mint_a.as_ref(), mint_b.as_ref()],
        amm::ID,
    );
    let vault_a = pda(&[b"vault_a", pool.as_ref()], amm::ID);
    let vault_b = pda(&[b"vault_b", pool.as_ref()], amm::ID);
    let lp_mint = pda(&[b"lp_mint", pool.as_ref()], amm::ID);
    let locked_lp = pda(&[b"locked_lp", pool.as_ref()], amm::ID);
    let user_lp = pda(&[b"initial_lp", pool.as_ref(), owner.as_ref()], amm::ID);
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            amm::accounts::CreatePool {
                creator: owner,
                config,
                admin: owner,
                mint_a,
                mint_b,
                pool,
                vault_a,
                vault_b,
                lp_mint,
                locked_lp,
                creator_lp: user_lp,
                source_a: user_a,
                source_b: user_b,
                token_program: spl_token_interface::ID,
                system_program: Pubkey::default(),
            },
            amm::instruction::CreatePool {
                amount_a: 1_000_000,
                amount_b: 4_000_000,
                min_shares: 1_999_000,
                deadline: 2_000,
            },
        ),
    )
    .unwrap();
    assert_eq!(balance(&svm, user_lp), 1_999_000);
    assert_eq!(balance(&svm, locked_lp), 1_000);
    let liquidity = || amm::accounts::Liquidity {
        user: owner,
        pool,
        mint_a,
        mint_b,
        vault_a,
        vault_b,
        lp_mint,
        user_a,
        user_b,
        user_lp,
        token_program: spl_token_interface::ID,
    };
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            liquidity(),
            amm::instruction::AddLiquidity {
                shares: 2_000_000,
                max_a: 1_000_000,
                max_b: 4_000_000,
                deadline: 2_000,
            },
        ),
    )
    .unwrap();
    let swap = || amm::accounts::Swap {
        user: owner,
        pool,
        config,
        mint_a,
        mint_b,
        vault_a,
        vault_b,
        user_a,
        user_b,
        token_program: spl_token_interface::ID,
    };
    let before = (balance(&svm, vault_a), balance(&svm, vault_b));
    assert!(send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            swap(),
            amm::instruction::Swap {
                a_to_b: true,
                input: 10_000,
                min_output: u64::MAX,
                deadline: 2_000
            }
        )
    )
    .is_err());
    assert_eq!((balance(&svm, vault_a), balance(&svm, vault_b)), before);
    assert!(send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            swap(),
            amm::instruction::Swap {
                a_to_b: true,
                input: 10_000,
                min_output: 1,
                deadline: 999
            }
        )
    )
    .is_err());
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            swap(),
            amm::instruction::Swap {
                a_to_b: true,
                input: 10_000,
                min_output: 1,
                deadline: 2_000,
            },
        ),
    )
    .unwrap();
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            swap(),
            amm::instruction::Swap {
                a_to_b: false,
                input: 20_000,
                min_output: 1,
                deadline: 2_000,
            },
        ),
    )
    .unwrap();
    let p: amm::state::Pool = state(&svm, pool);
    assert_eq!((p.protocol_fees_a, p.protocol_fees_b), (6, 12));
    let reserve_a = balance(&svm, vault_a) - p.protocol_fees_a;
    let reserve_b = balance(&svm, vault_b) - p.protocol_fees_b;
    assert!(
        u128::from(reserve_a) * u128::from(reserve_b)
            >= u128::from(before.0) * u128::from(before.1)
    );
    let thief = Keypair::new();
    svm.airdrop(&thief.pubkey(), 1_000_000_000).unwrap();
    let thief_a = tokens(&mut svm, mint_a, thief.pubkey(), 0);
    let thief_b = tokens(&mut svm, mint_b, thief.pubkey(), 0);
    assert!(send(
        &mut svm,
        &thief,
        ix(
            amm::ID,
            amm::accounts::CollectProtocolFees {
                admin: thief.pubkey(),
                config,
                pool,
                mint_a,
                mint_b,
                vault_a,
                vault_b,
                destination_a: thief_a,
                destination_b: thief_b,
                token_program: spl_token_interface::ID
            },
            amm::instruction::CollectProtocolFees {}
        )
    )
    .is_err());
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            amm::accounts::CollectProtocolFees {
                admin: owner,
                config,
                pool,
                mint_a,
                mint_b,
                vault_a,
                vault_b,
                destination_a: user_a,
                destination_b: user_b,
                token_program: spl_token_interface::ID,
            },
            amm::instruction::CollectProtocolFees {},
        ),
    )
    .unwrap();
    assert_eq!(balance(&svm, vault_a), reserve_a);
    assert_eq!(balance(&svm, vault_b), reserve_b);
    let shares = balance(&svm, user_lp);
    send(
        &mut svm,
        &user,
        ix(
            amm::ID,
            liquidity(),
            amm::instruction::RemoveLiquidity {
                shares,
                min_a: 1,
                min_b: 1,
                deadline: 2_000,
            },
        ),
    )
    .unwrap();
    assert_eq!(balance(&svm, user_lp), 0);
    assert!(balance(&svm, vault_a) > 0 && balance(&svm, vault_b) > 0);
    assert_eq!(
        Mint::unpack(&svm.get_account(&lp_mint).unwrap().data)
            .unwrap()
            .supply,
        1_000
    );
}
