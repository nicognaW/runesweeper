// Generation time on the web: the same benchmark as `cargo run --release --example solver`, for
// every Mode laid on the first click, run by Node's V8 on the web build's wasm
// (scripts/web-build.sh, so the same optimisation the site ships).
//   node scripts/wasm-bench.mjs [dist/web] [fields per level] [seed] [modes, comma-separated]
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [dir = 'dist/web', fields = '1000', seed = '1', only] = process.argv.slice(2);
const glue = await import(pathToFileURL(resolve(dir, 'app.js')));
await glue.default({ module_or_path: await readFile(resolve(dir, 'app_bg.wasm')) });

for (const mode of glue.generated_modes().filter((m) => !only || only.split(',').includes(m))) {
  console.log(`${mode} on wasm (Node ${process.versions.node}), first click anywhere, ${fields} fields per level, seed ${seed}`);
  console.log('level'.padEnd(13), 'accepted'.padStart(9), 'p50 ms'.padStart(9), 'p99 ms'.padStart(9), 'max ms'.padStart(9));
  for (const [name, w, h, mines] of [['Beginner', 9, 9, 10], ['Intermediate', 16, 16, 40], ['Expert', 30, 16, 99]]) {
    const [n, attempts, p50, p99, max] = glue.bench_generation(mode, w, h, mines, Number(fields), BigInt(seed));
    // Max Guess searches rather than redraws: it reports its steps per field instead.
    const accepted = mode === 'max-guess' ? `${(attempts / n).toFixed(0)} steps` : `${(100 * n / attempts).toFixed(2)}%`;
    console.log(name.padEnd(13), accepted.padStart(9), p50.toFixed(2).padStart(9), p99.toFixed(2).padStart(9), max.toFixed(2).padStart(9));
  }
  console.log();
}
