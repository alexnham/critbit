/** Preserve nested Kit/RPC errors instead of hiding them behind estimation wrappers. */
export function describeError(error: unknown): string {
  const messages: string[] = [];
  const logs: string[] = [];
  const seen = new Set<object>();
  function visit(value: unknown, depth = 0) {
    if (depth > 8 || value == null) return;
    if (typeof value !== 'object') { if (typeof value === 'string') messages.push(value); return; }
    if (seen.has(value)) return;
    seen.add(value);
    const node = value as {message?:unknown;cause?:unknown;context?:unknown;data?:unknown;logs?:unknown};
    if (typeof node.message === 'string') messages.push(node.message);
    if (Array.isArray(node.logs)) logs.push(...node.logs.filter((line):line is string=>typeof line==='string'));
    visit(node.cause,depth+1);
    visit(node.context,depth+1);
    visit(node.data,depth+1);
  }
  visit(error);
  if (!messages.length) messages.push(String(error));
  const uniqueLogs = [...new Set(logs)];
  const uniqueMessages = [...new Set(messages)];
  // Put the actual cause first; retain all outer error details below it.
  const result = [uniqueMessages.at(-1)!,...uniqueMessages.slice(0,-1).reverse()];
  const details = [...result,...uniqueLogs].join('\n');
  if (/insufficient (funds|lamports)|insufficient balance/i.test(details)) {
    result.unshift('Check balances in the connected wallet: it needs the full token deposits plus unwrapped SOL for rent and fees. Native SOL does not count as WSOL.');
  }
  if (/AccountNotInitialized|account.*not.*initialized/i.test(details)) {
    result.unshift('A required account is not initialized. Check the named account in the logs; initialize the AMM configuration first if it is config.');
  }
  if (uniqueLogs.length) result.push('\nSimulation logs:',...uniqueLogs);
  return result.join('\n\n');
}
