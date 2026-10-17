import {
  address, AccountRole, appendTransactionMessageInstructions, assertIsTransactionWithinSizeLimit,
  compileTransaction, createClient, createSolanaRpc, createTransactionMessage, estimateAndSetResourceLimitsFactory,
  estimateResourceLimitsFactory, fillTransactionMessageProvisoryResourceLimits, getAddressEncoder,
  getBase64Encoder, getBase64Decoder, getBase64EncodedWireTransaction, getProgramDerivedAddress,
  getSignatureFromTransaction, setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners, type TransactionMessageBytesBase64, type Instruction, type TransactionSigner,
} from '@solana/kit';
import { walletSigner } from '@solana/kit-plugin-wallet';
import { findAssociatedTokenPda, getCreateAssociatedTokenIdempotentInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
import { decodeAccount, encodeInstruction, IDLS, integer, json, type Kind, type Ix } from './idl';

export const walletClient = createClient().use(walletSigner({ chain: 'solana:devnet' }));
export const DEFAULT_RPC = 'https://api.devnet.solana.com';
export const SYSTEM = '11111111111111111111111111111111';
const DEVNET_GENESIS = 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG';
export type ContextFields = { admin: string; pool: string; config: string; mint_a: string; mint_b: string; mint: string; character_id: string };
export const EMPTY_CONTEXT: ContextFields = { admin:'', pool:'', config:'', mint_a:'', mint_b:'', mint:'', character_id:'1' };
export const makeRpc = (url: string) => createSolanaRpc(url);
const encoder = getAddressEncoder();
const utf8 = new TextEncoder();
export async function pda(program: string, seeds: (string | Uint8Array)[]) {
  const [key] = await getProgramDerivedAddress({ programAddress: address(program), seeds: seeds.map(s => typeof s === 'string' ? utf8.encode(s) : s) });
  return key;
}
const keyBytes = (s: string) => encoder.encode(address(s)) as Uint8Array;
async function findAccount(url: string, kind: Kind, key: string) {
  const result = await makeRpc(url).getAccountInfo(address(key), { encoding:'base64', commitment:'confirmed' }).send();
  if (!result.value) return null;
  if (result.value.owner !== IDLS[kind].address) throw new Error('Account is not owned by the selected Critbit program.');
  return decodeAccount(IDLS[kind], new Uint8Array(getBase64Encoder().encode(result.value.data[0])));
}
export async function loadAccount(url: string, kind: Kind, key: string) {
  const account = await findAccount(url,kind,key);
  if (!account) throw new Error('Account does not exist on this RPC. Initialize it or check the address.');
  return account;
}
export async function verifyNetwork(url: string) {
  const rpc = makeRpc(url);
  if (await rpc.getGenesisHash().send() !== DEVNET_GENESIS) throw new Error('This workbench is configured for devnet. Enter a devnet RPC endpoint.');
  const result = await rpc.getMultipleAccounts([address(IDLS.amm.address), address(IDLS.staking.address)], { encoding:'base64', commitment:'confirmed' }).send();
  if (result.value.some(a => !a?.executable)) throw new Error('One or both configured programs are not deployed on this RPC.');
  return true;
}
async function userToken(url: string, mint: string, owner: string) {
  const rpc = makeRpc(url);
  const result = await rpc.getTokenAccountsByOwner(address(owner), { mint:address(mint) }, { encoding:'jsonParsed', commitment:'confirmed' }).send();
  const accounts = result.value.map(a => {
    const info = a.account.data.parsed.info as { tokenAmount: { amount: string }; state: string };
    return { address: a.pubkey, balance: BigInt(info.tokenAmount.amount), state:info.state, program:a.account.owner };
  }).filter(a => a.program === TOKEN_PROGRAM_ADDRESS && a.state !== 'frozen');
  accounts.sort((a,b) => a.balance > b.balance ? -1 : a.balance < b.balance ? 1 : 0);
  if (accounts[0]) return accounts[0].address;
  return (await findAssociatedTokenPda({ mint: address(mint), owner: address(owner), tokenProgram:TOKEN_PROGRAM_ADDRESS }))[0];
}
export async function autofill(url: string, kind: Kind, ix: Ix, context: ContextFields, args: Record<string,string>, owner: string) {
  const id = IDLS[kind].address;
  const c = { ...context };
  const values = { ...args };
  const warnings: string[] = [];
  const messages: string[] = [];
  // Token-account discovery is optional: its failure must not hide derived PDAs.
  const tokenFor = async (mint: string) => {
    try { return await userToken(url,mint,owner); }
    catch {
      warnings.push(`Could not look up your token accounts for ${mint}. Using its associated token address; check your devnet RPC and that this is a token mint.`);
      return (await findAssociatedTokenPda({mint:address(mint),owner:address(owner),tokenProgram:TOKEN_PROGRAM_ADDRESS}))[0];
    }
  };
  const accounts: Record<string,string> = { token_program:TOKEN_PROGRAM_ADDRESS, system_program:SYSTEM, owner, user:owner, funder:owner, creator:owner, admin:owner };
  const admin = c.admin || owner;
  const derive = (...seeds: (string | Uint8Array)[]) => pda(id, seeds);
  const isNew = ix.name === 'create_pool' || ix.name === 'initialize_pool';
  if (kind === 'amm') {
    c.config = ix.name === 'initialize_config' ? await derive('config',keyBytes(owner)) : c.config || await derive('config',keyBytes(admin));
    accounts.config = c.config;
    if (ix.name !== 'initialize_config') {
      if (c.pool && !isNew) {
        const decoded = await loadAccount(url, kind, c.pool);
        if (decoded.type !== 'Pool') throw new Error('Choose an AMM Pool account.');
        Object.assign(c, Object.fromEntries(Object.entries(decoded.fields).filter(([k]) => ['config','mint_a','mint_b'].includes(k))));
        Object.assign(accounts, decoded.fields);
      }
      if (!c.mint_a || !c.mint_b) throw new Error('Enter both token mints, or load an existing pool.');
      let ab = keyBytes(c.mint_a), bb = keyBytes(c.mint_b);
      const order = ab.findIndex((v,i) => v !== bb[i]);
      if (order < 0) throw new Error('Choose two different token mints.');
      if (ab[order] > bb[order]) {
        [c.mint_a,c.mint_b] = [c.mint_b,c.mint_a];
        [ab,bb] = [bb,ab];
        for (const [a,b] of [['amount_a','amount_b'],['max_a','max_b'],['min_a','min_b']]) {
          if (a in values || b in values) [values[a],values[b]] = [values[b] ?? '',values[a] ?? ''];
        }
        if ('a_to_b' in values) values.a_to_b = values.a_to_b === 'true' ? 'false' : 'true';
        messages.push('Sorted token A/B into the program’s required order and moved any matching amounts/direction with them.');
      }
      c.pool = c.pool && !isNew ? c.pool : await derive('pool',keyBytes(c.config),ab,bb);
      Object.assign(accounts, { pool:c.pool, config:c.config, mint_a:c.mint_a, mint_b:c.mint_b,
        vault_a:await derive('vault_a',keyBytes(c.pool)), vault_b:await derive('vault_b',keyBytes(c.pool)),
        lp_mint:await derive('lp_mint',keyBytes(c.pool)), locked_lp:await derive('locked_lp',keyBytes(c.pool)),
        creator_lp:await derive('initial_lp',keyBytes(c.pool),keyBytes(owner)) });
      const config = await findAccount(url, kind, c.config);
      if (config && config.type !== 'Config') throw new Error('Choose an AMM Config account.');
      accounts.admin = config ? String(config.fields.admin) : admin;
      if (!config) warnings.push('Your AMM configuration has not been initialized. Use Initialize configuration with the admin wallet before creating the pool. The derived addresses are filled below.');
      const [a,b] = await Promise.all([tokenFor(c.mint_a),tokenFor(c.mint_b)]);
      Object.assign(accounts,{source_a:a,source_b:b,user_a:a,user_b:b,destination_a:a,destination_b:b});
      if (ix.accounts.some(a => a.name === 'user_lp')) accounts.user_lp = await tokenFor(accounts.lp_mint);
    }
  } else {
    if (c.pool && !isNew) {
      const decoded = await loadAccount(url, kind, c.pool);
      if (decoded.type !== 'RewardPool') throw new Error('Choose a staking RewardPool account.');
      c.mint = String(decoded.fields.mint); c.admin = String(decoded.fields.admin);
      Object.assign(accounts, decoded.fields);
    }
    if (!c.mint) throw new Error('Enter your staking token mint, or load a reward pool.');
    c.pool = c.pool && !isNew ? c.pool : await derive('reward_pool',keyBytes(isNew ? owner : admin),keyBytes(c.mint));
    Object.assign(accounts, { pool:c.pool, mint:c.mint, admin:isNew ? owner : c.admin || admin,
      stake_vault:await derive('stake_vault',keyBytes(c.pool)), reward_vault:await derive('reward_vault',keyBytes(c.pool)) });
    const characterId = ix.name === 'register_character' ? args.character_id : c.character_id;
    c.character_id = characterId || '1';
    const charBytes = new Uint8Array(4); new DataView(charBytes.buffer).setUint32(0,Number(integer(characterId || '1','u32')),true);
    accounts.character = await derive('character',keyBytes(c.pool),charBytes);
    accounts.position = await derive('position',keyBytes(c.pool),keyBytes(owner),keyBytes(accounts.character));
    if (ix.accounts.some(a => ['user_tokens','source','destination'].includes(a.name))) {
      const token = await tokenFor(c.mint);
      Object.assign(accounts,{user_tokens:token,source:token,destination:token});
    }
  }
  return { context:c, args:values, warnings, messages, accounts:Object.fromEntries(ix.accounts.map(a => [a.name, String(accounts[a.name] || a.address || '')])) };
}
export function buildInstruction(kind: Kind, ix: Ix, args: Record<string,string>, accounts: Record<string,string>, signer: TransactionSigner): Instruction {
  return { programAddress:address(IDLS[kind].address), data:encodeInstruction(ix,args), accounts:ix.accounts.map(a => {
    const key = address(accounts[a.name] || a.address || '');
    if (a.signer && key !== signer.address) throw new Error(`${a.name} must sign. Connect that wallet; this UI uses one signing wallet per transaction.`);
    return a.signer ? { address:key, role:a.writable ? AccountRole.WRITABLE_SIGNER : AccountRole.READONLY_SIGNER, signer } : { address:key, role:a.writable ? AccountRole.WRITABLE : AccountRole.READONLY };
  }) };
}
export async function ataInstructions(url: string, ix: Ix, accounts: Record<string,string>, signer: TransactionSigner) {
  const mappings: Record<string,string> = { source:'mint',destination:'mint',user_tokens:'mint',source_a:'mint_a',source_b:'mint_b',user_a:'mint_a',user_b:'mint_b',destination_a:'mint_a',destination_b:'mint_b',user_lp:'lp_mint' };
  const list: Instruction[] = []; const seen = new Set<string>();
  for (const account of ix.accounts) {
    const mintName = mappings[account.name]; if (!mintName) continue;
    const key = accounts[account.name]; if (seen.has(key)) continue; seen.add(key);
    const mint = address(accounts[mintName]);
    const canonical = (await findAssociatedTokenPda({ mint,owner:signer.address,tokenProgram:TOKEN_PROGRAM_ADDRESS }))[0];
    if (key !== canonical) continue;
    const existing = await makeRpc(url).getAccountInfo(canonical, {encoding:'base64'}).send();
    if (!existing.value) list.push(getCreateAssociatedTokenIdempotentInstruction({payer:signer,owner:signer.address,mint,ata:canonical,tokenProgram:TOKEN_PROGRAM_ADDRESS}));
  }
  return list;
}
export async function prepare(url: string, kind: Kind, ix: Ix, args: Record<string,string>, accounts: Record<string,string>, createAtas: boolean) {
  await verifyNetwork(url);
  const connected = walletClient.wallet.getState().connected;
  if (!connected?.signer) throw new Error('Connect a signing wallet first.');
  const signer = connected.signer;
  const version = connected.supportedTransactionVersions.has(1) ? 1 : connected.supportedTransactionVersions.has(0) ? 0 : 'legacy';
  const instructions = [...(createAtas ? await ataInstructions(url,ix,accounts,signer) : []),buildInstruction(kind,ix,args,accounts,signer)];
  const rpc = makeRpc(url);
  const {value:blockhash} = await rpc.getLatestBlockhash({commitment:'confirmed'}).send();
  const draft = fillTransactionMessageProvisoryResourceLimits(appendTransactionMessageInstructions(instructions,setTransactionMessageLifetimeUsingBlockhash(blockhash,setTransactionMessageFeePayerSigner(signer,createTransactionMessage({version})))));
  const estimate = estimateResourceLimitsFactory({rpc});
  const message = await estimateAndSetResourceLimitsFactory(async m => {
    const limits = await estimate(m);
    return {...limits,computeUnitLimit:Math.min(1_400_000,Math.ceil(limits.computeUnitLimit*1.15)),...(limits.loadedAccountsDataSizeLimit !== undefined ? {loadedAccountsDataSizeLimit:Math.ceil((limits.loadedAccountsDataSizeLimit+32768)/32768)*32768}: {})};
  })(draft);
  const compiled = compileTransaction(message);
  assertIsTransactionWithinSizeLimit(compiled);
  const simulation = await rpc.simulateTransaction(getBase64EncodedWireTransaction(compiled),{encoding:'base64',sigVerify:false,commitment:'confirmed'}).send();
  if (simulation.value.err) throw new Error(`Simulation failed: ${json(simulation.value.err)}\n${simulation.value.logs?.join('\n')}`);
  const fee = await rpc.getFeeForMessage(getBase64Decoder().decode(compiled.messageBytes) as TransactionMessageBytesBase64,{commitment:'confirmed'}).send();
  return {message,url,kind,action:ix.name,args:{...args},accounts:{...accounts},owner:signer.address,version,fee:fee.value,logs:simulation.value.logs ?? [],compute:simulation.value.unitsConsumed,createdAt:Date.now(),ataCount:instructions.length-1};
}
export type Prepared = Awaited<ReturnType<typeof prepare>>;
export async function submit(plan: Prepared, onSignature: (s: string) => void) {
  if (Date.now()-plan.createdAt > 45_000) throw new Error('Review expired. Simulate again for a fresh blockhash.');
  if (walletClient.wallet.getState().connected?.account.address !== plan.owner) throw new Error('Wallet changed. Simulate again.');
  const signed = await signTransactionMessageWithSigners(plan.message);
  const rpc = makeRpc(plan.url);
  const signature = getSignatureFromTransaction(signed);
  onSignature(signature);
  try {
    await rpc.sendTransaction(getBase64EncodedWireTransaction(signed),{encoding:'base64',skipPreflight:false,preflightCommitment:'confirmed',maxRetries:3n}).send();
  } catch (e) { throw new Error(`Broadcast did not return success. Check the transaction in the explorer before retrying. ${e instanceof Error ? e.message : String(e)}`); }
  for (let i=0;i<45;i++) {
    const result = await rpc.getSignatureStatuses([signature]).send(); const status = result.value[0];
    if (status?.err) throw new Error(`Transaction failed: ${json(status.err)}`);
    if (status?.confirmationStatus === 'confirmed' || status?.confirmationStatus === 'finalized') return signature;
    await new Promise(resolve => setTimeout(resolve,1500));
  }
  throw new Error('Submitted, but confirmation timed out. Check the explorer before retrying.');
}
export async function inspectMint(url: string, key: string) {
  const account = await makeRpc(url).getAccountInfo(address(key),{encoding:'jsonParsed',commitment:'confirmed'}).send();
  if (!account.value || account.value.owner !== TOKEN_PROGRAM_ADDRESS || !('parsed' in account.value.data) || account.value.data.parsed.type !== 'mint') throw new Error('Enter a classic SPL Token mint.');
  return account.value.data.parsed.info as {decimals:number;supply:string};
}
