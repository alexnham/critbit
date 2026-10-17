import {describe,it,expect} from 'vitest';
import {describeError} from './errors';
describe('simulation error reporting',()=>{
  it('unwraps non-enumerable causes and preserves Kit simulation logs',()=>{
    const cause=new Error('Instruction 2 failed: custom program error 1');
    const error=Object.assign(new Error('Resource estimation failed',{cause}),{context:{logs:['Program Tokenkeg invoked','Program log: Error: insufficient funds']}});
    const result=describeError(error);
    expect(result).toContain('Instruction 2 failed');
    expect(result).toContain('Program log: Error: insufficient funds');
    expect(result).toContain('Native SOL does not count as WSOL');
    expect(result.indexOf('Instruction 2 failed')).toBeLessThan(result.indexOf('Resource estimation failed'));
  });
  it('preserves RPC preflight data logs and Anchor account names',()=>{
    const error=new Error('RPC request failed',{cause:{message:'Simulation failed',data:{logs:['AnchorError caused by account: config. Error Code: AccountNotInitialized.']}}});
    expect(describeError(error)).toContain('initialize the AMM configuration');
    expect(describeError(error)).toContain('account: config');
  });
  it('handles circular causes and ordinary errors',()=>{
    const e=new Error('Plain failure');e.cause=e;
    expect(describeError(e)).toBe('Plain failure');
    expect(describeError('Wallet rejected signing')).toBe('Wallet rejected signing');
  });
});
