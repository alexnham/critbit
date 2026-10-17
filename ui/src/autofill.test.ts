import {afterEach,describe,expect,it,vi} from 'vitest';
import {address,getAddressDecoder,getAddressEncoder} from '@solana/kit';
import {TOKEN_PROGRAM_ADDRESS,findAssociatedTokenPda} from '@solana-program/token';
import {autofill,EMPTY_CONTEXT,pda} from './chain';
import {IDLS} from './idl';
const owner='GbDLTR4JK4EdZAtjXqAVDx3vWg2BwwEiESxsLqhDNEiG';
const mintA=getAddressDecoder().decode(new Uint8Array(32).fill(1));
const mintB=getAddressDecoder().decode(new Uint8Array(32).fill(2));
const ix=IDLS.amm.instructions.find(i=>i.name==='create_pool')!;
function rpcMock(failTokens=false){
  vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
    const body=JSON.parse(typeof init?.body==='string'?init.body:await (input as Request).text());
    const result=body.method==='getAccountInfo'?{context:{slot:1},value:null}:{context:{slot:1},value:[]};
    return new Response(JSON.stringify(failTokens&&body.method==='getTokenAccountsByOwner'?{jsonrpc:'2.0',id:body.id,error:{code:-32602,message:'Token account lookup unavailable'}}:{jsonrpc:'2.0',id:body.id,result}),{headers:{'Content-Type':'application/json'}});
  }));
}
afterEach(()=>vi.unstubAllGlobals());
describe('create-pool autofill',()=>{
 it('fills every account even before the admin configuration exists',async()=>{
  rpcMock();
  const filled=await autofill('https://rpc.test','amm',ix,{...EMPTY_CONTEXT,mint_a:mintA,mint_b:mintB},{amount_a:'1000',amount_b:'5000'},owner);
  expect(Object.keys(filled.accounts)).toHaveLength(ix.accounts.length);
  expect(Object.values(filled.accounts).every(Boolean)).toBe(true);
  expect(filled.warnings.join(' ')).toContain('Initialize configuration');
  expect(filled.accounts.source_a).toBe((await findAssociatedTokenPda({mint:mintA,owner:address(owner),tokenProgram:TOKEN_PROGRAM_ADDRESS}))[0]);
 });
 it('sorts reversed mints and preserves matching deposit amounts',async()=>{
  rpcMock();
  const filled=await autofill('https://rpc.test','amm',ix,{...EMPTY_CONTEXT,mint_a:mintB,mint_b:mintA},{amount_a:'1000000000',amount_b:'1234',min_shares:'1'},owner);
  expect(filled.context.mint_a).toBe(mintA);expect(filled.context.mint_b).toBe(mintB);
  expect(filled.args).toEqual({amount_a:'1234',amount_b:'1000000000',min_shares:'1'});
  expect(filled.messages.join(' ')).toContain('Sorted token');
  const encode=getAddressEncoder();
  expect(filled.accounts.pool).toBe(await pda(IDLS.amm.address,['pool',new Uint8Array(encode.encode(address(filled.accounts.config))),new Uint8Array(encode.encode(mintA)),new Uint8Array(encode.encode(mintB))]));
 });
 it('falls back to associated accounts if token discovery fails',async()=>{
  rpcMock(true);
  const filled=await autofill('https://rpc.test','amm',ix,{...EMPTY_CONTEXT,mint_a:mintA,mint_b:mintB},{},owner);
  expect(filled.accounts.source_a).toBeTruthy();expect(filled.accounts.source_b).toBeTruthy();
  expect(filled.warnings.join(' ')).toContain('Could not look up your token accounts');
 });
 it('rejects identical mints with an actionable message',async()=>{
  rpcMock();
  await expect(autofill('https://rpc.test','amm',ix,{...EMPTY_CONTEXT,mint_a:mintA,mint_b:mintA},{},owner)).rejects.toThrow('two different token mints');
 });
});
