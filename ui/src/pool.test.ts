import {afterEach,describe,it,expect,vi} from 'vitest';
import {address,getAddressDecoder,getAddressEncoder,getBase64Decoder} from '@solana/kit';
import {TOKEN_PROGRAM_ADDRESS} from '@solana-program/token';
import {quote,units,ratio,readPool,swapArgs,liquidityArgs,WSOL,CRIT,type PoolSnapshot} from './pool';
import {IDLS} from './idl';
afterEach(()=>vi.unstubAllGlobals());
describe('pool arithmetic',()=>{
 it('prepares proportional liquidity with safe deposit/withdrawal limits',()=>{
  const pool={a:{decimals:9,reserve:10000000000n},b:{decimals:9,reserve:100000000000000n},lpSupply:1000000000000n,walletLp:999999999000n} as PoolSnapshot;
  const added=liquidityArgs(pool,true,'1','0.5');
  expect(added.a).toBe(10000000n);expect(added.b).toBe(100000000000n);
  expect(added.args.max_a).toBe('10050000');expect(added.args.shares).toBe('1000000000');
  const removed=liquidityArgs(pool,false,'1','0.5');expect(removed.args.min_a).toBe('9950000');
  expect(()=>liquidityArgs(pool,false,'1000','0.5')).toThrow('exceeds');
 });
 it('matches on-chain fees, exact output, and slippage rounding',()=>{
  const result=quote(100000000n,10000000000n,100000000000000n,30n,50n);
  expect(result.fee).toBe(300000n);
  expect(result.output).toBe(987158034397n);
  expect(result.minOutput).toBe(result.output*9950n/10000n);
  expect((10000000000n+100000000n-result.fee)*(100000000000000n-result.output)).toBeGreaterThanOrEqual(10000000000n*100000000000000n);
 });
 it('handles large integers and fees that round up',()=>{
  expect(units(100000000000000n,9)).toBe('100,000');
  expect(units(999999999000n,9)).toBe('999.999999');
  expect(ratio(100000000000000n,10000000000n)).toBe('10,000');
  expect(quote(101n,10000n,10000000n,30n,100n).fee).toBe(1n);
  expect(()=>quote(1n,100n,100n,30n,50n)).toThrow();
  expect(()=>quote(1n,0n,100n,30n,50n)).toThrow();
 });
 it('converts human swap amounts to the selected token decimals',()=>{
  const pool={a:{decimals:9,reserve:10000000000n},b:{decimals:6,reserve:100000000000n},feeBps:30n} as PoolSnapshot;
  const forward=swapArgs(pool,true,'0.1','0.5');
  expect(forward.args.input).toBe('100000000');
  expect(forward.args.min_output).toBe(forward.quote.minOutput.toString());
  expect(swapArgs(pool,false,'12.5','1').args.input).toBe('12500000');
  expect(()=>swapArgs(pool,true,'0.0000000001','0.5')).toThrow();
 });
});
function fixture(wrongAuthority=false){
 const enc=getAddressEncoder();const dec=getAddressDecoder();const key=(n:number)=>dec.decode(new Uint8Array(32).fill(n));
 const pool=key(1),config=key(2),vaultA=key(3),vaultB=key(4),lp=key(5),admin=key(6);
 const discriminator=IDLS.amm.accounts.find(a=>a.name==='Pool')!.discriminator;
 const poolData=Uint8Array.from([...discriminator,...[config,WSOL,CRIT,vaultA,vaultB,lp].flatMap(k=>[...enc.encode(address(k))]),...new Uint8Array(16),254]);
 const configData=new Uint8Array(45);configData.set(IDLS.amm.accounts.find(a=>a.name==='Config')!.discriminator);configData.set(enc.encode(admin),8);new DataView(configData.buffer).setUint16(40,30,true);new DataView(configData.buffer).setUint16(42,2000,true);
 const mint=(supply:bigint)=>{const data=new Uint8Array(82);data[44]=9;data[45]=1;new DataView(data.buffer).setBigUint64(36,supply,true);return data;};
 const token=(m:typeof WSOL|string,amount:bigint)=>{const data=new Uint8Array(165);data.set(enc.encode(m as Parameters<typeof enc.encode>[0]));data.set(enc.encode(wrongAuthority?admin:pool),32);new DataView(data.buffer).setBigUint64(64,amount,true);data[108]=1;return data;};
 const account=(data:Uint8Array,owner:string)=>({data:[getBase64Decoder().decode(data),'base64'],owner,lamports:10000000,executable:false,rentEpoch:0});
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const body=JSON.parse(typeof init?.body==='string'?init.body:await(input as Request).text());
  const result=body.method==='getAccountInfo'?{context:{slot:100},value:account(poolData,IDLS.amm.address)}:body.method==='getMultipleAccounts'?{context:{slot:101},value:[account(configData,IDLS.amm.address),account(mint(10000000000n),TOKEN_PROGRAM_ADDRESS),account(mint(100000000000000n),TOKEN_PROGRAM_ADDRESS),account(token(WSOL,10000000000n),TOKEN_PROGRAM_ADDRESS),account(token(CRIT,100000000000000n),TOKEN_PROGRAM_ADDRESS),account(mint(1000000000000n),TOKEN_PROGRAM_ADDRESS)]}:[];
  return new Response(JSON.stringify({jsonrpc:'2.0',id:body.id,result}),{headers:{'Content-Type':'application/json'}});
 }));return pool;
}
describe('validated pool snapshot',()=>{
 it('decodes vaults, mint decimals, fee settings and LP supply',async()=>{
  const key=fixture();const data=await readPool('https://rpc.test',key);
  expect(data.a.reserve).toBe(10000000000n);expect(data.b.reserve).toBe(100000000000000n);
  expect(data.a.symbol).toBe('WSOL');expect(data.b.symbol).toBe('CB');
  expect(data.feeBps).toBe(30n);expect(data.protocolBps).toBe(2000n);
  expect(data.lpSupply).toBe(1000000000000n);expect(data.walletLp).toBeNull();expect(data.slot).toBe(101n);
 });
 it('rejects a vault with an unrelated authority',async()=>{
  const key=fixture(true);await expect(readPool('https://rpc.test',key)).rejects.toThrow('authority does not match');
 });
});
