# Critbit program workbench

Local React + Vite UI for every instruction in both deployed Anchor programs.

## Start

```sh
cd program-building/ui
npm install
npm run dev
```

Open http://127.0.0.1:5173 in a browser with a Solana Wallet Standard extension
(such as Phantom). Set the wallet to devnet and connect it. A Codex browser preview
may not have your wallet extension; open the same URL in your regular browser.

Devnet program addresses come from the generated IDLs:

- AMM: `2UJP3iqx5Pn6SWuZarFfhWcxYSUuvQPNbXmjArSrWqaA`
- Staking: `BgT4LsMdYp7q73t2UtkM2soKgXJHdXo2joQoaAM9UFxN`

`Connection settings` accepts a custom **devnet** RPC. Before preparing a
transaction the UI checks the devnet genesis hash and both executable accounts.
No wallet files, private keys, signing server, or backend are used.

## Pool inspector

The home screen loads your devnet pool (`CCdFQHLFD6tXS9HgZEN9NQ3d3piLKvhdRuhsgKmEFVRW`)
and remembers the last successfully inspected address locally. Use Find pools to
list pools owned by the AMM program, or paste a Pool address and select Inspect pool.

- Reserve balances exclude uncollected protocol fees. Token decimals are read from
  each mint; token ownership, vault authority, and account layouts are validated.
- Spot prices are reserve ratios, not external market prices or dollar valuations.
- LP supply and connected-wallet holdings include LP tokens held outside ATAs.
- Swap previews accept human token amounts and calculate exact base-unit input,
  rounded fees, output, and minimum output using the on-chain constant-product math.
- Add/remove liquidity previews accept human LP amounts and compute proportional
  deposits/withdrawals with slippage limits. Review loads the matching instruction
  form and account addresses; it does not sign or submit anything.
- Recent activity links to actual pool transactions; it is not labeled as swap
  volume, revenue, or a historical price chart.

Refresh explicitly to get a new snapshot. Related reserve/mint/config accounts are
read together; wallet holdings and activity are separate reads. No price oracle is
used. Advanced controls retain all original AMM and staking instructions.

## Using an instruction

1. Select AMM or Character staking, then choose an instruction. On narrow screens,
   use the Instruction dropdown.
2. For an existing pool, enter its address. For a new pool, supply the token mint(s)
   and admin/configuration as needed. Leaving admin blank uses the connected wallet.
3. Click **Autofill accounts**. The UI reads pool/config state and derives vault,
   mint, character, and position PDAs. Existing wallet token accounts with the
   highest balance are preferred, including the creator's initial LP account.
4. Enter arguments. Amounts are raw base units, not displayed token quantities.
   The amount converter handles decimals exactly. LP amounts use 9 decimals.
   Minimum output/withdrawal amounts must be provided explicitly.
5. Click **Simulate & review**. Review parameters, token mints, recipient accounts,
   fees, and logs. **Sign & send transaction** opens the wallet and broadcasts only
   after signing. A signature is displayed even when broadcast confirmation is
   uncertain; check the explorer before retrying.
6. Load a pool, character, or position in **Account inspector** to read updated
   state. Closing a position removes its account, so it can no longer be loaded.

Missing canonical associated token accounts can be created in the transaction.
This does not mint tokens or top up an empty funding account. Other token-account
addresses can be entered manually in Instruction accounts. Account constraints
and the simulation validate them before signing.

AMM creation uses one wallet for both config admin and liquidity creator. The
program also supports separate signers, but coordinating multiple wallets is
outside this single-wallet UI. Autofill sorts mint A/B by public-key bytes when needed, moves their matching
amounts/direction with them, and displays a notice. Missing AMM configuration or
failed token-account discovery is reported beside Autofill without hiding the
derived addresses. Initialize configuration before submitting Create pool.

The staking character ID in setup context is used for existing positions. When
registering a character, the instruction's character_id argument is authoritative.
A position must be opened before staking. Reward funding must cover obligations
through the fixed campaign end. Rates are simple annual 5% / 7.5% / 10%.

## Instruction coverage

AMM: initialize_config, create_pool, add_liquidity, remove_liquidity, swap,
collect_protocol_fees.

Staking: initialize_pool, register_character, fund_rewards, open_position, stake,
unstake, claim, reclaim_unused_rewards, close_position.

## Development

```sh
npm test
npm run build
npm run sync-idl  # after rebuilding/redeploying the Anchor programs
```

The IDL codec supports the primitive instruction types and fixed-layout account
structures present in these programs. Unsupported types fail explicitly. Tests
verify coverage, discriminator/argument bytes, account order and permissions,
integer precision, signer constraints, and account decoding.

The UI uses Solana Kit and Wallet Standard. Version 1 is used when the wallet
supports it, with v0/legacy fallback for older wallets. Resource limits are
estimated from simulation with headroom. The review expires after 45 seconds;
changing wallet, form, or connection invalidates it.

Build and browser checks do not authorize a real transaction. End-to-end wallet
signing and devnet mutations must be performed by the connected wallet owner.
