import {address,getBase64Encoder,getAddressDecoder} from '@solana/kit';
import {TOKEN_PROGRAM_ADDRESS} from '@solana-program/token';
import {IDLS,decodeAccount,decimalToUnits} from './idl';
import {loadAccount,makeRpc,type ContextFields} from './chain';
export const WSOL='So11111111111111111111111111111111111111112';
export const CRIT='AQ5dCAKAvBFJyZx41YHCJEBuwPySHACBRqJX1SRshAWx';
export const INITIAL_POOL='CCdFQHLFD6tXS9HgZEN9NQ3d3piLKvhdRuhsgKmEFVRW';
export function units(value:bigint,decimals:number,precision=6){
 const scale=10n**BigInt(decimals);const whole=value/scale;const rest=(value%scale).toString().padStart(decimals,'0').slice(0,precision).replace(/0+$/,'');
 const formatted=whole.toString().replace(/\B(?=(\d{3})+(?!\d))/g,',');
 return formatted+(rest?'.'+rest:'');
}
export function ratio(numerator:bigint,denominator:bigint,precision=9){
 if(denominator<=0n)return '—';
 return units(numerator*10n**BigInt(precision)/denominator,precision,precision);
}
export function quote(input:bigint,reserveIn:bigint,reserveOut:bigint,feeBps:bigint,slippageBps:bigint){
 if(input<=0n||reserveIn<=0n||reserveOut<=0n)throw new Error('Enter a positive amount into a funded pool.');
 if(feeBps<0n||feeBps>=10000n||slippageBps<0n||slippageBps>=10000n)throw new Error('Invalid fee or slippage.');
 const fee=(input*feeBps+9999n)/10000n;const net=input-fee;
 const output=net*reserveOut/(reserveIn+net);
 if(output<=0n)throw new Error('This amount is too small to receive one output base unit.');
 return {output,fee,minOutput:output*(10000n-slippageBps)/10000n,impactBps:net*10000n/(reserveIn+net)};
}
export type Asset={mint:string;symbol:string;decimals:number;balance:bigint;reserve:bigint;fees:bigint;vault:string};
export type PoolSnapshot={key:string;context:ContextFields;a:Asset;b:Asset;lpMint:string;lpSupply:bigint;walletLp:bigint|null;feeBps:bigint;protocolBps:bigint;slot:bigint;loadedAt:number;raw:Record<string,unknown>;activity:{signature:string;blockTime:bigint|null;err:unknown}[];warnings:string[]};
export async function discoverPools(url:string){
 const result=await makeRpc(url).getProgramAccounts(address(IDLS.amm.address),{encoding:'base64',commitment:'confirmed',filters:[{dataSize:217n}]}).send();
 return result.filter(a=>{try{return decodeAccount(IDLS.amm,new Uint8Array(getBase64Encoder().encode(a.account.data[0]))).type==='Pool';}catch{return false;}}).map(a=>a.pubkey);
}
export async function readPool(url:string,key:string,owner?:string):Promise<PoolSnapshot>{
 const decoded=await loadAccount(url,'amm',key);
 if(decoded.type!=='Pool')throw new Error('This address is not an AMM pool. Paste the Pool address, not a token mint or vault.');
 const f=decoded.fields;const rpc=makeRpc(url);
 const keys=['config','mint_a','mint_b','vault_a','vault_b','lp_mint'].map(k=>address(String(f[k])));
 const snapshot=await rpc.getMultipleAccounts(keys,{encoding:'base64',commitment:'confirmed'}).send();
 const data=snapshot.value.map((a,i)=>{
  if(!a)throw new Error(`Pool account ${keys[i]} is missing.`);
  const expected=i===0?IDLS.amm.address:TOKEN_PROGRAM_ADDRESS;
  if(a.owner!==expected)throw new Error(`Unexpected owner for pool account ${keys[i]}.`);
  return new Uint8Array(getBase64Encoder().encode(a.data[0]));
 });
 const config=decodeAccount(IDLS.amm,data[0]);if(config.type!=='Config')throw new Error('Invalid pool configuration.');
 const pubkey=getAddressDecoder();
 function token(index:number,mintIndex:number,fees:bigint):Asset{
  const mint=data[mintIndex],vault=data[index];
  if(mint.length!==82||mint[45]!==1||vault.length!==165||vault[108]!==1)throw new Error('Unexpected or frozen mint/vault account.');
  if(pubkey.decode(vault.slice(0,32))!==keys[mintIndex]||pubkey.decode(vault.slice(32,64))!==key)throw new Error('Vault mint or authority does not match the pool.');
  const balance=new DataView(vault.buffer,vault.byteOffset,vault.byteLength).getBigUint64(64,true);
  if(fees>balance)throw new Error('Protocol fees exceed the vault balance.');
  const mintKey=keys[mintIndex];
  return {mint:mintKey,symbol:mintKey===WSOL?'WSOL':mintKey===CRIT?'CB':`${mintKey.slice(0,4)}…${mintKey.slice(-4)}`,decimals:mint[44],balance,reserve:balance-fees,fees,vault:keys[index]};
 }
 const a=token(3,1,BigInt(String(f.protocol_fees_a))),b=token(4,2,BigInt(String(f.protocol_fees_b)));
 const lp=data[5];if(lp.length!==82||lp[45]!==1||lp[44]!==9)throw new Error('Invalid LP mint.');
 const lpSupply=new DataView(lp.buffer,lp.byteOffset,lp.byteLength).getBigUint64(36,true);
 const warnings:string[]=[];
 const [walletLp,activity]=await Promise.all([
  owner?rpc.getTokenAccountsByOwner(address(owner),{mint:keys[5]},{encoding:'jsonParsed',commitment:'confirmed'}).send().then(r=>r.value.reduce((sum,a)=>sum+BigInt((a.account.data.parsed.info as {tokenAmount:{amount:string}}).tokenAmount.amount),0n)).catch(()=>{warnings.push('Wallet LP balance could not be loaded.');return null;}):Promise.resolve(null),
  rpc.getSignaturesForAddress(address(key),{limit:8,commitment:'confirmed'}).send().then(r=>r.map(t=>({signature:t.signature,blockTime:t.blockTime,err:t.err}))).catch(()=>{warnings.push('Recent activity is unavailable on this RPC.');return [];})
 ]);
 return {key,context:{admin:String(config.fields.admin),config:keys[0],mint_a:a.mint,mint_b:b.mint,pool:key,mint:'',character_id:'1'},a,b,lpMint:keys[5],lpSupply,walletLp,feeBps:BigInt(String(config.fields.fee_bps)),protocolBps:BigInt(String(config.fields.protocol_share_bps)),slot:snapshot.context.slot,loadedAt:Date.now(),raw:f,activity,warnings};
}
export function swapArgs(pool:PoolSnapshot,aToB:boolean,amount:string,slippage:string){
 const input=aToB?pool.a:pool.b,output=aToB?pool.b:pool.a;
 const base=BigInt(decimalToUnits(amount,input.decimals));
 const slip=BigInt(decimalToUnits(slippage,2));
 const q=quote(base,input.reserve,output.reserve,pool.feeBps,slip);
 return {quote:q,args:{a_to_b:String(aToB),input:base.toString(),min_output:q.minOutput.toString(),deadline:String(Math.floor(Date.now()/1000)+600)}};
}
export function liquidityArgs(pool:PoolSnapshot,adding:boolean,amount:string,slippage:string){
 const shares=BigInt(decimalToUnits(amount,9)),slip=BigInt(decimalToUnits(slippage,2));
 if(shares<=0n||pool.lpSupply<=0n)throw new Error('Enter a positive LP token amount.');
 if(slip>=10000n)throw new Error('Slippage must be below 100%.');
 if(!adding&&pool.walletLp!==null&&shares>pool.walletLp)throw new Error('This amount exceeds your wallet’s LP balance.');
 const a=adding?(shares*pool.a.reserve+pool.lpSupply-1n)/pool.lpSupply:shares*pool.a.reserve/pool.lpSupply;
 const b=adding?(shares*pool.b.reserve+pool.lpSupply-1n)/pool.lpSupply:shares*pool.b.reserve/pool.lpSupply;
 if(!a||!b)throw new Error('Amount is too small for both token reserves.');
 const args:Record<string,string>={shares:shares.toString(),deadline:String(Math.floor(Date.now()/1000)+600)};
 if(adding){args.max_a=((a*(10000n+slip)+9999n)/10000n).toString();args.max_b=((b*(10000n+slip)+9999n)/10000n).toString();}
 else{args.min_a=(a*(10000n-slip)/10000n).toString();args.min_b=(b*(10000n-slip)/10000n).toString();}
 return {a,b,args};
}
