import type { Kind } from './idl';
export const ACTIONS: Record<Kind, { name: string; title: string; description: string; admin?: boolean }[]> = {
  amm: [
    {name:'initialize_config',title:'Initialize configuration',description:'Set immutable swap fees for your admin wallet.',admin:true},
    {name:'create_pool',title:'Create a pool',description:'Deposit both tokens and receive the initial liquidity shares. Your wallet must be the configuration admin.',admin:true},
    {name:'add_liquidity',title:'Add liquidity',description:'Choose exactly how many LP shares to mint and cap each token deposit.'},
    {name:'remove_liquidity',title:'Remove liquidity',description:'Burn your LP shares and withdraw both tokens with minimum amounts.'},
    {name:'swap',title:'Swap tokens',description:'Trade an exact input amount. Minimum output protects against slippage.'},
    {name:'collect_protocol_fees',title:'Collect protocol fees',description:'Transfer accumulated protocol fees to the admin wallet.',admin:true},
  ],
  staking: [
    {name:'initialize_pool',title:'Initialize reward pool',description:'Create a staking campaign with separate principal and reward vaults.',admin:true},
    {name:'register_character',title:'Register a character',description:'Enable a character with the 1,000 / 5,000 / 10,000 token staking tiers.',admin:true},
    {name:'fund_rewards',title:'Fund rewards',description:'Deposit tokens into the reward vault. Anyone can fund the campaign.'},
    {name:'open_position',title:'Open character position',description:'Create your empty staking position for a registered character.'},
    {name:'stake',title:'Stake & unlock',description:'Allocate tokens to this character. The resulting balance determines its level.'},
    {name:'unstake',title:'Withdraw stake',description:'Withdraw principal immediately. The character level follows the remaining stake.'},
    {name:'claim',title:'Claim rewards',description:'Collect all payable rewards. Your staked principal stays in place.'},
    {name:'reclaim_unused_rewards',title:'Reclaim surplus',description:'Recover only unreserved reward tokens. Earned and promised rewards remain protected.',admin:true},
    {name:'close_position',title:'Close position',description:'Recover rent from an empty position. Principal and whole base-unit rewards must be zero; fractional dust is forfeited.'},
  ],
};
export const ARG_HELP: Record<string,string> = {
  fee_bps:'30 = 0.3% of swap input.', protocol_share_bps:'2,000 = 20% of the trading fee.',
  duration_seconds:'31,536,000 = 365 days. Maximum: five years.', character_id:'The game’s numeric character ID.',
  deadline:'Unix timestamp. Autofill uses ten minutes from now.', shares:'LP base units (9 decimals).',
  min_shares:'Minimum initial LP base units you will accept.', amount_a:'Token A base units.',amount_b:'Token B base units.',
  max_a:'Maximum token A base units to deposit.',max_b:'Maximum token B base units to deposit.',
  min_a:'Minimum token A base units to receive.',min_b:'Minimum token B base units to receive.',
  input:'Input token base units.',min_output:'Minimum output token base units to receive.',amount:'Token base units; use the converter below.',
};
export const DEFAULT_ARGS: Record<string,string> = { fee_bps:'30', protocol_share_bps:'2000', duration_seconds:'31536000', character_id:'1', a_to_b:'true' };
