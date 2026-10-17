import { describe,it,expect } from 'vitest';
import { createNoopSigner, address, getAddressEncoder, AccountRole } from '@solana/kit';
import { IDLS,encodeInstruction,integer,decimalToUnits,decodeAccount } from './idl';
import { ACTIONS } from './catalog';
import { buildInstruction } from './chain';
const wallet=address('GbDLTR4JK4EdZAtjXqAVDx3vWg2BwwEiESxsLqhDNEiG');
describe('Anchor instruction coverage',()=>{
  for(const kind of ['amm','staking'] as const){
    it(`exposes every ${kind} instruction and preserves account order/roles`,()=>{
      expect(ACTIONS[kind].map(a=>a.name).sort()).toEqual(IDLS[kind].instructions.map(a=>a.name).sort());
      for(const ix of IDLS[kind].instructions){
        const args=Object.fromEntries(ix.args.map(a=>[a.name,a.type==='bool'?'true':'42']));
        const accounts=Object.fromEntries(ix.accounts.map(a=>[a.name,a.address||wallet]));
        const built=buildInstruction(kind,ix,args,accounts,createNoopSigner(wallet));
        expect(built.programAddress).toBe(IDLS[kind].address);
        expect(Array.from(built.data!.slice(0,8))).toEqual(ix.discriminator);
        expect(built.accounts?.map(a=>a.address)).toEqual(ix.accounts.map(a=>accounts[a.name]));
        expect(built.accounts?.map(a=>a.role)).toEqual(ix.accounts.map(a=>a.signer?(a.writable?AccountRole.WRITABLE_SIGNER:AccountRole.READONLY_SIGNER):(a.writable?AccountRole.WRITABLE:AccountRole.READONLY)));
      }
    });
  }
  it('matches Rust little-endian swap ABI',()=>{
    const ix=IDLS.amm.instructions.find(i=>i.name==='swap')!;
    const bytes=encodeInstruction(ix,{a_to_b:'true',input:'18446744073709551615',min_output:'256',deadline:'-1'});
    expect(Array.from(bytes.slice(8,10))).toEqual([1,255]);
    expect(Array.from(bytes.slice(9,17))).toEqual(Array(8).fill(255));
    expect(Array.from(bytes.slice(17,25))).toEqual([0,1,0,0,0,0,0,0]);
    expect(Array.from(bytes.slice(25))).toEqual(Array(8).fill(255));
  });
  it('rejects unknown signing authority before asking a wallet',()=>{
    const ix=IDLS.amm.instructions.find(i=>i.name==='initialize_config')!;
    expect(()=>buildInstruction('amm',ix,{fee_bps:'30',protocol_share_bps:'2000'},{admin:IDLS.amm.address,config:wallet,system_program:'11111111111111111111111111111111'},createNoopSigner(wallet))).toThrow('must sign');
  });
});
describe('exact amount handling',()=>{
  it('does not round amounts through JavaScript numbers',()=>{
    expect(decimalToUnits('12345678901.123456789',9)).toBe('12345678901123456789');
    expect(decimalToUnits('1000',6)).toBe('1000000000');
    expect(()=>decimalToUnits('1.0000001',6)).toThrow();
    expect(()=>decimalToUnits('1e3',6)).toThrow();
    expect(()=>integer('18446744073709551616','u64')).toThrow();
    expect(()=>integer('-1','u64')).toThrow();
    expect(()=>integer('1.5','u64')).toThrow();
  });
});
describe('account decoding',()=>{
  it('validates the discriminator and decodes a Rust Config layout',()=>{
    const account=IDLS.amm.accounts.find(a=>a.name==='Config')!;
    const data=Uint8Array.from([...account.discriminator,...getAddressEncoder().encode(wallet),30,0,208,7,255]);
    expect(decodeAccount(IDLS.amm,data)).toEqual({type:'Config',fields:{admin:wallet,fee_bps:'30',protocol_share_bps:'2000',bump:'255'}});
    expect(()=>decodeAccount(IDLS.amm,data.slice(0,12))).toThrow('truncated');
    expect(()=>decodeAccount(IDLS.staking,data)).toThrow('does not match');
  });
  it('decodes character tiers with nested fixed arrays',()=>{
    const account=IDLS.staking.accounts.find(a=>a.name==='CharacterConfig')!;
    const tier=(level:number,stake:bigint,rate:number)=>{
      const data=new Uint8Array(11);const view=new DataView(data.buffer);data[0]=level;view.setBigUint64(1,stake,true);view.setUint16(9,rate,true);return [...data];
    };
    const data=Uint8Array.from([...account.discriminator,...getAddressEncoder().encode(wallet),1,0,0,0,...tier(1,1000n,500),...tier(2,5000n,750),...tier(3,10000n,1000),254]);
    const decoded=decodeAccount(IDLS.staking,data);
    expect(decoded.fields.tiers).toEqual([{level:'1',required_stake_base_units:'1000',reward_rate_bps:'500'},{level:'2',required_stake_base_units:'5000',reward_rate_bps:'750'},{level:'3',required_stake_base_units:'10000',reward_rate_bps:'1000'}]);
  });
});
