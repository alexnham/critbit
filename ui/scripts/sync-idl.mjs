import { copyFileSync } from 'node:fs';
for (const name of ['amm', 'staking']) {
  copyFileSync(new URL(`../../target/idl/critbit_${name}.json`, import.meta.url), new URL(`../src/idl/${name}.json`, import.meta.url));
}
console.log('Synced both generated Anchor IDLs.');
