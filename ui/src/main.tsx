import React, { useEffect, useRef, useState, useSyncExternalStore } from 'react';
import { createRoot } from 'react-dom/client';
import { ACTIONS, ARG_HELP, DEFAULT_ARGS } from './catalog';
import { IDLS, decimalToUnits, json, label, type Kind } from './idl';
import { autofill, DEFAULT_RPC, EMPTY_CONTEXT, inspectMint, loadAccount, prepare, submit, verifyNetwork, walletClient, type ContextFields, type Prepared } from './chain';
import './style.css';
import { describeError } from './errors';
import { PoolInspector } from './PoolInspector';
import type { PoolSnapshot } from './pool';

const short = (key:string) => `${key.slice(0,5)}…${key.slice(-5)}`;
const explorer = (key:string, tx=false) => `https://explorer.solana.com/${tx?'tx':'address'}/${key}?cluster=devnet`;
const defaults = () => ({...DEFAULT_ARGS,deadline:String(Math.floor(Date.now()/1000)+600)});
function App() {
  const wallet = useSyncExternalStore(walletClient.wallet.subscribe,walletClient.wallet.getState);
  const [kind,setKind] = useState<Kind>('amm');
  const [poolView,setPoolView] = useState(true);
  const [action,setAction] = useState('swap');
  const [rpc,setRpc] = useState(DEFAULT_RPC);
  const [settings,setSettings] = useState(false);
  const [network,setNetwork] = useState('Checking programs…');
  const [contexts,setContexts] = useState<Record<Kind,ContextFields>>({amm:{...EMPTY_CONTEXT},staking:{...EMPTY_CONTEXT}});
  const context = contexts[kind];
  const [args,setArgs] = useState<Record<string,string>>(defaults);
  const [accounts,setAccounts] = useState<Record<string,string>>({});
  const [plan,setPlan] = useState<Prepared|null>(null);
  const [busy,setBusy] = useState('');
  const [error,setError] = useState('');
  const [notice,setNotice] = useState('');
  const [signature,setSignature] = useState('');
  const [confirmed,setConfirmed] = useState(false);
  const [createAtas,setCreateAtas] = useState(true);
  const [accountDetails,setAccountDetails] = useState(false);
  const [inspectKey,setInspectKey] = useState('');
  const [inspected,setInspected] = useState<Awaited<ReturnType<typeof loadAccount>>|null>(null);
  const [unitAmount,setUnitAmount] = useState('1000');
  const [decimals,setDecimals] = useState('6');
  const [mintInfo,setMintInfo] = useState('');
  const [walletMenu,setWalletMenu] = useState(false);
  const revision = useRef(0);
  const ix = IDLS[kind].instructions.find(i=>i.name===action)!;
  const descriptor = ACTIONS[kind].find(i=>i.name===action)!;
  const owner = wallet.connected?.account.address;
  const invalidate = () => { revision.current++; setPlan(null); setError(''); setNotice(''); };
  useEffect(()=>{ invalidate(); setAccounts({}); },[owner]);
  useEffect(()=>{ let active=true; setNetwork('Checking programs…'); verifyNetwork(rpc).then(()=>{if(active)setNetwork('Programs online');}).catch(e=>{if(active)setNetwork(e.message);}); return ()=>{active=false;}; },[rpc]);
  function select(nextKind:Kind,nextAction:string) {
    invalidate(); setPoolView(false); setKind(nextKind);setAction(nextAction);setArgs(defaults());setAccounts({});setInspected(null);setInspectKey('');setSignature('');setConfirmed(false);setAccountDetails(false);setMintInfo('');
  }
  function updateContext(key:keyof ContextFields,value:string) {
    invalidate();setAccounts({});setContexts(prev=>({...prev,[kind]:{...prev[kind],[key]:value}}));
  }
  async function run(title:string, fn:()=>Promise<void>) { setBusy(title);setError('');try{await fn();}catch(e){setError(describeError(e));setPlan(null);}finally{setBusy('');} }
  async function fill() {
    if(!owner)throw new Error('Connect your wallet to derive accounts.');
    const current = revision.current;
    const result = await autofill(rpc,kind,ix,context,args,owner);
    if (current !== revision.current) return;
    setContexts(prev=>({...prev,[kind]:result.context}));setAccounts(result.accounts);
    setArgs({...result.args,...(ix.args.some(a=>a.name==='deadline')?{deadline:String(Math.floor(Date.now()/1000)+600)}:{})});
    setNotice(['Accounts filled. Expand Instruction accounts below to review them.',...result.messages,...result.warnings].join(' '));
    setAccountDetails(true);
  }
  async function simulate() {
    const current = revision.current;
    if (!Object.keys(accounts).length) throw new Error('Use Autofill accounts, or enter the instruction accounts below.');
    const result=await prepare(rpc,kind,ix,args,accounts,createAtas);
    if(current===revision.current)setPlan(result);
  }
  async function send() {
    if(!plan)return;
    setSignature('');setConfirmed(false);
    await submit(plan,s=>{setSignature(s);setPlan(null);});setConfirmed(true);setNotice('Transaction confirmed on devnet. Load an account to see the updated state.');
  }
  async function inspect() {
    const key=inspectKey||accounts.position||accounts.pool||context.pool||accounts.config;
    if(!key)throw new Error('Enter an account address to inspect.');
    setInspectKey(key);setInspected(await loadAccount(rpc,kind,key));
  }
  function useInspectedPool(pool:PoolSnapshot,nextAction:string,nextArgs:Record<string,string>={}) {
    select('amm',nextAction);
    setContexts(prev=>({...prev,amm:pool.context}));
    const values={...defaults(),...nextArgs};setArgs(values);
    if(!owner){setNotice('Connect your wallet, then use Autofill accounts. The inspected pool is selected.');return;}
    const current=revision.current;
    void run('Preparing pool accounts',async()=>{
      const instruction=IDLS.amm.instructions.find(i=>i.name===nextAction)!;
      const result=await autofill(rpc,'amm',instruction,pool.context,values,owner);
      if(current!==revision.current)return;
      setAccounts(result.accounts);setArgs(result.args);setAccountDetails(false);
      setNotice(['Pool accounts loaded. Review the amounts, then simulate.',...result.warnings].join(' '));
    });
  }
  let converted='';try{converted=decimalToUnits(unitAmount,Number(decimals));}catch{converted='Check amount and decimals';}
  return <div className="app-shell">
    <aside className="sidebar">
      <a className="brand" href="#" onClick={e=>e.preventDefault()}><span className="pixel-mark" aria-hidden="true"><i/><i/><i/><i/></span><span>critbit<span className="brand-sub">Program workbench</span></span></a>
      <nav aria-label="Program navigation">
        <button className={`program-tab pool-nav ${kind==='amm'&&poolView?'active':''}`} disabled={!!busy} onClick={()=>{invalidate();setKind('amm');setPoolView(true);}}>◉ <span>Pool inspector</span></button>
        <div className="nav-caption">Advanced controls</div>
        {(['amm','staking'] as Kind[]).map(k=><div key={k} className="nav-group">
          <button className={`program-tab ${kind===k&&!poolView?'active':''}`} disabled={!!busy} onClick={()=>select(k,k==='amm'?'swap':'stake')}><span>{k==='amm'?'◈':'✦'}</span>{k==='amm'?'AMM':'Character staking'}<span className="count">{ACTIONS[k].length}</span></button>
          {kind===k&&!poolView&&<div className="action-nav">{ACTIONS[k].map(a=><button key={a.name} disabled={!!busy} className={action===a.name?'selected':''} onClick={()=>select(k,a.name)}>{a.title}{a.admin&&<span title="Requires admin wallet">↟</span>}</button>)}</div>}
        </div>)}
      </nav>
      <div className="sidebar-bottom"><span className="network-dot"/> Devnet workspace<p>Two programs. One connected wallet.</p><button className="sidebar-link" onClick={()=>setSettings(!settings)} disabled={!!busy}>Connection settings</button></div>
    </aside>
    <main>
      <header className="topbar"><div className="breadcrumb">Workbench <span>/</span> {kind==='amm'?(poolView?'Pool inspector':'AMM controls'):'Character staking'}</div><div className="wallet-control"><span className="network-badge">Devnet</span><button className="connect" disabled={!!busy||['connecting','disconnecting'].includes(wallet.status)} onClick={()=>setWalletMenu(!walletMenu)}>{owner?short(owner):'Connect wallet'}</button>
      {walletMenu&&<div className="wallet-menu">{owner?<><strong>{wallet.connected?.wallet.name}</strong><code>{owner}</code><button onClick={()=>run('Disconnecting',async()=>{await walletClient.wallet.disconnect();setWalletMenu(false);})}>Disconnect</button></>:<><strong>Choose a wallet</strong>{wallet.wallets.length?wallet.wallets.map((w,i)=><button key={`${w.name}-${i}`} onClick={()=>run('Connecting wallet',async()=>{await walletClient.wallet.connect(w);setWalletMenu(false);})}>{w.icon&&<img src={w.icon} alt=""/>}{w.name}</button>):<p>No wallet detected. Open this page in a browser with a Solana wallet extension, set to devnet.</p>}</>}</div>}</div></header>
      <div className="workspace">
        {kind==='amm'&&poolView&&settings&&<section className="panel connection-panel"><h2>Connection</h2><label>Devnet RPC<input value={rpc} onChange={e=>{invalidate();setRpc(e.target.value);}} spellCheck={false}/></label><p className="hint">{network}</p></section>}
        {kind==='amm'&&poolView?<PoolInspector rpc={rpc} owner={owner} disabled={!!busy} onUse={useInspectedPool}/>:<>
        <label className="mobile-actions">Instruction<select disabled={!!busy} value={action} onChange={e=>select(kind,e.target.value)}>{ACTIONS[kind].map(a=><option key={a.name} value={a.name}>{a.title}</option>)}</select></label><div className="page-heading"><div><h1>{kind==='amm'?'Liquidity & trading':'Characters & rewards'}</h1><p>{kind==='amm'?'Manage your constant-product pools, trades, and fees.':'Stake your token to unlock Critbit characters and earn rewards.'}</p></div><span className="program-status"><span className={`status-dot ${network==='Programs online'?'online':''}`}/>{network==='Programs online'?'Program online':'RPC status'}</span></div>
        {(settings||network!=='Programs online')&&<section className="panel connection-panel"><h2>Connection</h2><label>Devnet RPC<input value={rpc} disabled={!!busy} onChange={e=>{invalidate();setRpc(e.target.value);}} spellCheck={false}/></label><p className={network==='Programs online'?'hint':'warning'}>{network}</p><div className="program-links">{(['amm','staking'] as Kind[]).map(k=><a key={k} href={explorer(IDLS[k].address)} target="_blank" rel="noreferrer">{k==='amm'?'AMM':'Staking'}: {IDLS[k].address}</a>)}</div></section>}
        {kind==='staking'&&<div className="tier-strip">{[{level:1,stake:'1,000',rate:'5%'},{level:2,stake:'5,000',rate:'7.5%'},{level:3,stake:'10,000',rate:'10%'}].map(t=><div key={t.level}><span className="tier-symbol">{'✦'.repeat(t.level)}</span><div><strong>Level {t.level}</strong><span>{t.stake} tokens staked</span></div><b>{t.rate}<small>APR</small></b></div>)}</div>}
        <section className="panel context-panel"><div className="section-title"><h2>{kind==='amm'?'Pool context':'Character context'}</h2><span>Load an existing pool or enter setup details</span></div><fieldset disabled={!!busy}><div className="context-grid">
          <label className="span-two">{kind==='amm'?'AMM pool address':'Reward pool address'}<input placeholder="Paste an existing pool address" value={context.pool} onChange={e=>updateContext('pool',e.target.value.trim())} spellCheck={false}/></label>
          <label>{kind==='amm'?'Configuration admin':'Pool admin'}<input placeholder="Connected wallet by default" value={context.admin} onChange={e=>updateContext('admin',e.target.value.trim())} spellCheck={false}/></label>
          {kind==='amm'?<><label>Token A mint<input placeholder="First mint, sorted by public-key bytes" value={context.mint_a} onChange={e=>updateContext('mint_a',e.target.value.trim())} spellCheck={false}/></label><label>Token B mint<input placeholder="Second mint" value={context.mint_b} onChange={e=>updateContext('mint_b',e.target.value.trim())} spellCheck={false}/></label><label>Configuration address<input placeholder="Derived from admin if empty" value={context.config} onChange={e=>updateContext('config',e.target.value.trim())} spellCheck={false}/></label></>:<><label className="span-two">Staking token mint<input placeholder="Your classic SPL Token mint" value={context.mint} onChange={e=>updateContext('mint',e.target.value.trim())} spellCheck={false}/></label><label>Character ID<input inputMode="numeric" value={context.character_id} onChange={e=>updateContext('character_id',e.target.value)}/></label></>}
          </div><div className="context-actions"><p>Vaults and positions are derived automatically. Existing token accounts are preferred.</p><button className="secondary" onClick={()=>{invalidate();run('Loading accounts',fill);}}>{busy==='Loading accounts'?'Filling accounts…':'Autofill accounts'}</button></div>
          {!owner&&<p className="hint">Connect your wallet before autofilling accounts.</p>}
          {busy&&<div role="status" className="notice"><span className="spinner"/>{busy}…</div>}
          {error&&<div role="alert" className="error-box"><strong>Could not complete the action</strong><pre>{error}</pre></div>}
          {notice&&!busy&&<div role="status" className="notice">{notice}</div>}
          </fieldset></section>
        <div className="work-grid"><section className="panel instruction-panel"><div className="instruction-heading"><span className="action-icon">{kind==='amm'?'⇄':'✦'}</span><div><h2>{descriptor.title}</h2><p>{descriptor.description}</p></div>{descriptor.admin&&<span className="admin-badge">Admin</span>}</div>
          <form onSubmit={e=>{e.preventDefault();setPlan(null);run('Simulating transaction',simulate);}}><fieldset disabled={!!busy}>
          <div className="argument-grid">{ix.args.map(arg=><label key={arg.name}>{label(arg.name)}{arg.type==='bool'?<select value={args[arg.name]} onChange={e=>{invalidate();setArgs({...args,[arg.name]:e.target.value});}}><option value="true">Token A → Token B</option><option value="false">Token B → Token A</option></select>:<input required inputMode="numeric" placeholder="Enter whole base units" value={args[arg.name]||''} onChange={e=>{invalidate();setArgs({...args,[arg.name]:e.target.value});}}/>}<span className="hint">{ARG_HELP[arg.name]}</span></label>)}</div>
          {!ix.args.length&&<div className="no-args">This instruction uses the selected accounts and needs no amount parameters.</div>}
          <details className="account-details" open={accountDetails} onToggle={e=>setAccountDetails(e.currentTarget.open)}><summary>Instruction accounts <span>{ix.accounts.length}</span></summary><p className="hint">Autofill first. Override an address only if you need a specific token account.</p><div className="account-fields">{ix.accounts.map(a=><label key={a.name}><span>{label(a.name)} {a.signer&&<small>Signer</small>}</span><input value={accounts[a.name]||a.address||''} placeholder="Use Autofill accounts" readOnly={!!a.address} onChange={e=>{invalidate();setAccounts({...accounts,[a.name]:e.target.value.trim()});}} spellCheck={false}/></label>)}</div></details>
          <label className="checkbox"><input type="checkbox" checked={createAtas} onChange={e=>{invalidate();setCreateAtas(e.target.checked);}}/>Create missing associated token accounts when needed</label>
          <div className="submit-row"><span>{owner?'Simulate first. Sign only after review.':'Connect a wallet to prepare this instruction.'}</span><button className="primary" type="submit" disabled={!owner}>{busy==='Simulating transaction'?'Simulating…':'Simulate & review'}</button></div>
          </fieldset></form></section>
          <aside className="tools"><section className="panel inspector"><h2>Account inspector</h2><p>Read pool, character, or position state directly from devnet.</p><label>Account address<input value={inspectKey} placeholder="Defaults to the selected position or pool" onChange={e=>{setInspectKey(e.target.value.trim());setInspected(null);}} disabled={!!busy} spellCheck={false}/></label><button className="secondary full" disabled={!!busy} onClick={()=>run('Reading account',inspect)}>Load account</button>{inspected&&<div className="inspect-result"><strong>{inspected.type}</strong><a href={explorer(inspectKey)} target="_blank" rel="noreferrer">View in explorer ↗</a><pre>{json(inspected.fields)}</pre></div>}</section>
          <section className="panel converter"><h2>Amount converter</h2><p>Instruction amounts use base units. Convert token amounts without rounding.</p><div className="converter-grid"><label>Tokens<input value={unitAmount} onChange={e=>setUnitAmount(e.target.value)} inputMode="decimal"/></label><label>Decimals<input value={decimals} onChange={e=>setDecimals(e.target.value)} inputMode="numeric"/></label></div><output>{converted}</output><span className="hint">Base units</span><button className="text-button" disabled={!!busy} onClick={()=>run('Reading mint',async()=>{const mint=context.mint||context.mint_a;if(!mint)throw new Error('Enter a mint in the pool context first.');const info=await inspectMint(rpc,mint);setDecimals(String(info.decimals));setMintInfo(`Mint ${short(mint)} uses ${info.decimals} decimals.`);})}>Read decimals from {kind==='amm'?'mint A':'staking mint'}</button>{mintInfo&&<p className="hint">{mintInfo}</p>}</section>
          <div className="program-note"><span>Connected program</span><a href={explorer(IDLS[kind].address)} target="_blank" rel="noreferrer">{short(IDLS[kind].address)} ↗</a><small>Wallet signatures stay in your wallet.</small></div></aside></div>
        {signature&&<div className="signature"><strong>{confirmed?'Confirmed':'Broadcast attempted'}</strong><a href={explorer(signature,true)} target="_blank" rel="noreferrer">View transaction ↗</a><code>{signature}</code></div>}
        {plan&&<section className="panel review" aria-label="Transaction review"><div className="section-title"><h2>Review transaction</h2><span className="success-badge">Simulation passed</span></div><dl><div><dt>Action</dt><dd>{label(plan.action)}</dd></div><div><dt>Network</dt><dd>Devnet · {plan.version==='legacy'?'Legacy':`Version ${plan.version}`}</dd></div><div><dt>Fee payer</dt><dd>{plan.owner}</dd></div><div><dt>Estimated network fee</dt><dd>{plan.fee===null?'Unavailable':`${Number(plan.fee)/1e9} SOL`} <small>Account creation also requires rent.</small></dd></div><div><dt>Token account setup</dt><dd>{plan.ataCount} new associated account(s)</dd></div><div><dt>Compute units</dt><dd>{String(plan.compute??'Unavailable')}</dd></div></dl><div className="review-data"><details open><summary>Amounts & parameters</summary><pre>{json(plan.args)}</pre></details><details open><summary>Recipients & instruction accounts</summary><pre>{json(plan.accounts)}</pre></details><details><summary>Simulation logs</summary><pre>{plan.logs.join('\n')}</pre></details></div><div className="submit-row"><p>Review expires after 45 seconds. Signing submits this transaction.</p><button className="primary" disabled={!!busy} onClick={()=>run('Awaiting signature and confirmation',send)}>Sign & send transaction</button></div></section>}
        </>}
        <footer>Critbit development tools <span>{ACTIONS.amm.length+ACTIONS.staking.length} instructions available</span></footer>
      </div>
    </main>
  </div>;
}
createRoot(document.getElementById('root')!).render(<React.StrictMode><App/></React.StrictMode>);
