// No Guess generation time on the web: the same benchmark as `cargo run --release --example
// solver`, run by Node's V8 on the web build's wasm (scripts/web-build.sh, so the same
// optimisation the site ships).
//   node scripts/wasm-bench.mjs [dist/web] [fields per level] [seed]
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [dir = 'dist/web', fields = '1000', seed = '1'] = process.argv.slice(2);
const glue = await import(pathToFileURL(resolve(dir, 'app.js')));
await glue.default({ module_or_path: await readFile(resolve(dir, 'app_bg.wasm')) });

console.log(`No Guess on wasm (Node ${process.versions.node}), first click anywhere, ${fields} fields per level, seed ${seed}`);
console.log('level'.padEnd(13), 'accepted'.padStart(9), 'p50 ms'.padStart(9), 'p99 ms'.padStart(9), 'max ms'.padStart(9));
for (const [name, w, h, mines] of [['Beginner', 9, 9, 10], ['Intermediate', 16, 16, 40], ['Expert', 30, 16, 99]]) {
  const [n, attempts, p50, p99, max] = glue.bench_no_guess(w, h, mines, Number(fields), BigInt(seed));
  console.log(name.padEnd(13), `${(100 * n / attempts).toFixed(2)}%`.padStart(9), p50.toFixed(2).padStart(9), p99.toFixed(2).padStart(9), max.toFixed(2).padStart(9));
}
