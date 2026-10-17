import { getAddressDecoder } from '@solana/kit';
import amm from './idl/amm.json';
import staking from './idl/staking.json';
export type Kind = 'amm' | 'staking';
export type IdlType = string | { array: [IdlType, number] } | { defined: { name: string } };
export type Field = { name: string; type: IdlType };
export type Ix = { name: string; discriminator: number[]; accounts: { name: string; writable?: boolean; signer?: boolean; address?: string }[]; args: Field[] };
export type Idl = { address: string; instructions: Ix[]; accounts: { name: string; discriminator: number[] }[]; types: { name: string; type: { fields: Field[] } }[]; errors: { code: number; name: string; msg: string }[] };
export const IDLS: Record<Kind, Idl> = { amm: amm as Idl, staking: staking as Idl };
export const label = (s: string) => s.replaceAll('_', ' ').replace(/^./, c => c.toUpperCase());
export const json = (v: unknown) => JSON.stringify(v, (_, x) => typeof x === 'bigint' ? x.toString() : x, 2);
export function integer(value: string, type: string): bigint {
  if (!/^-?\d+$/.test(value.trim())) throw new Error(`Enter a whole number for ${type}.`);
  const n = BigInt(value); const signed = type.startsWith('i'); const bits = BigInt(type.slice(1));
  const min = signed ? -(1n << (bits - 1n)) : 0n;
  const max = signed ? (1n << (bits - 1n)) - 1n : (1n << bits) - 1n;
  if (n < min || n > max) throw new Error(`Value is outside the ${type} range.`);
  return n;
}
export function encodeInstruction(ix: Ix, values: Record<string, string>) {
  const bytes = [...ix.discriminator];
  for (const field of ix.args) {
    if (field.type === 'bool') {
      if (!['true','false'].includes(values[field.name])) throw new Error(`Choose ${field.name}.`);
      bytes.push(values[field.name] === 'true' ? 1 : 0); continue;
    }
    if (typeof field.type !== 'string' || !/^[ui](8|16|32|64|128)$/.test(field.type)) throw new Error('Unsupported IDL argument.');
    const bits = Number(field.type.slice(1)); let n = BigInt.asUintN(bits, integer(values[field.name] ?? '', field.type));
    for (let i = 0; i < bits / 8; i++) { bytes.push(Number(n & 255n)); n >>= 8n; }
  }
  return Uint8Array.from(bytes);
}
export function decodeAccount(idl: Idl, data: Uint8Array) {
  const account = idl.accounts.find(a => a.discriminator.every((b, i) => data[i] === b));
  if (!account) throw new Error('This account does not match the selected program IDL.');
  let offset = account.discriminator.length;
  function take(size: number) { if (offset + size > data.length) throw new Error('Account data is truncated.'); const b = data.slice(offset, offset + size); offset += size; return b; }
  function read(type: IdlType): unknown {
    if (typeof type !== 'string') {
      if ('array' in type) return Array.from({ length: type.array[1] }, () => read(type.array[0]));
      const def = idl.types.find(t => t.name === type.defined.name);
      if (!def) throw new Error('Unknown IDL type.');
      return Object.fromEntries(def.type.fields.map(f => [f.name, read(f.type)]));
    }
    if (type === 'pubkey') return getAddressDecoder().decode(take(32));
    if (type === 'bool') { const b = take(1)[0]; if (b > 1) throw new Error('Invalid bool.'); return b === 1; }
    if (!/^[ui](8|16|32|64|128)$/.test(type)) throw new Error('Unsupported account type.');
    const bits = Number(type.slice(1)); const bytes = take(bits / 8); let value = 0n;
    for (let i = bytes.length - 1; i >= 0; i--) value = (value << 8n) | BigInt(bytes[i]);
    return (type.startsWith('i') ? BigInt.asIntN(bits, value) : value).toString();
  }
  return { type: account.name, fields: read({ defined: { name: account.name } }) as Record<string, unknown> };
}
export function decimalToUnits(value: string, decimals: number) {
  if (!/^\d+(\.\d*)?$/.test(value) || !Number.isInteger(decimals) || decimals < 0 || decimals > 18) throw new Error('Enter a positive decimal amount and 0–18 decimals.');
  const [whole, fraction = ''] = value.split('.');
  if (fraction.length > decimals) throw new Error('Amount has more decimals than the mint.');
  return integer(whole + fraction.padEnd(decimals, '0'), 'u64').toString();
}
